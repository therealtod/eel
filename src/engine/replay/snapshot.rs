use serde_json::{Value, json};

use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::clue_type::ClueType;
use crate::game::variant::Variant;

const SUIT_NAMES: [&str; 5] = ["red", "yellow", "green", "blue", "purple"];
const SUIT_LETTERS: [char; 5] = ['r', 'y', 'g', 'b', 'p'];

fn card_id_to_string(card_id: usize) -> String {
    let suit_idx = card_id / 5;
    let rank = card_id % 5 + 1;
    format!("{}{}", SUIT_LETTERS[suit_idx], rank)
}

fn mask_to_inferred_string(mask: u64) -> String {
    (0..25)
        .filter(|&id| (mask >> id) & 1 != 0)
        .map(card_id_to_string)
        .collect::<String>()
}

/// Reconstruct positive and negative clue lists that reproduce `deck_empathy`.
///
/// Positive: clues where `deck_empathy ⊆ clue_mask` (applying them as a positive clue
/// narrows the full-deck mask toward `deck_empathy`).
/// Negative: clues where `deck_empathy ∩ clue_mask = ∅` (they exclude entire suit/rank groups).
///
/// The intersection of all positive clue masks, filtered by the complement of all negative
/// clue masks, equals `deck_empathy`.
fn reconstruct_clues(deck_empathy: u64, variant: &Variant) -> (Vec<String>, Vec<String>) {
    let mut positive = Vec::new();
    let mut negative = Vec::new();

    for suit_idx in 0..variant.number_of_suits as usize {
        let clue_mask = variant.empathy_by_clue(ClueType::Color, suit_idx).as_bits();
        if (deck_empathy & !clue_mask) == 0 {
            positive.push(SUIT_NAMES[suit_idx].to_string());
        } else if (deck_empathy & clue_mask) == 0 {
            negative.push(SUIT_NAMES[suit_idx].to_string());
        }
    }

    for rank in 1..=variant.stacks_size {
        let clue_mask = variant
            .empathy_by_clue(ClueType::Rank, rank as usize)
            .as_bits();
        if (deck_empathy & !clue_mask) == 0 {
            positive.push(rank.to_string());
        } else if (deck_empathy & clue_mask) == 0 {
            negative.push(rank.to_string());
        }
    }

    (positive, negative)
}

/// Serialise `game` as a `ScenarioJson`-compatible `serde_json::Value`.
///
/// The produced JSON can be written to `tests/scenarios/<name>/table_state.json` and loaded
/// by the helpers in `tests/common/mod.rs`.  `prior_actions` is always emitted empty — the
/// snapshot captures the state at one moment in time; history belongs to the replay harness.
///
/// **Caveats**:
/// - Convention state accumulated across turns (cohort linkage, hypothesis chains) is not fully
///   preserved.  `src/bin/snapshot.rs` always compares the engine recommendation before and after
///   serialisation and warns when they differ; fall back to the replay harness in those cases.
/// - `discard_pile` is serialised by card-id order (all R1 copies first, then R2, …), not by
///   the chronological order cards were discarded.  `TableState` stores only a count-per-card
///   multiset, so the original sequence is not recoverable.  Multi-set membership is preserved,
///   but discard timing is lost.
pub fn to_scenario_json(game: &KnowledgeAwareGameState) -> Value {
    let ts = &game.table_state;
    let tk = &game.team_knowledge;
    let sd = game.static_data();
    let variant = &sd.variant;
    debug_assert_eq!(
        variant.number_of_suits, 5,
        "snapshot serialisation hardcodes NO_VARIANT"
    );
    debug_assert_eq!(
        variant.stacks_size, 5,
        "snapshot serialisation hardcodes NO_VARIANT"
    );
    let num_players = sd.number_of_players as usize;
    let active = ts.active_player_index;

    let suits: Vec<Value> = (0..variant.number_of_suits as usize)
        .map(|s| json!(SUIT_NAMES[s]))
        .collect();

    let playing_stacks: Vec<Value> = (0..variant.number_of_suits as usize)
        .map(|s| {
            let size = ts.playing_stacks.stack_size(s) as usize;
            let cards: Vec<Value> = (0..size)
                .map(|rank_0| json!(card_id_to_string(s * variant.stacks_size as usize + rank_0)))
                .collect();
            json!(cards)
        })
        .collect();

    let mut discard: Vec<Value> = Vec::new();
    let num_unique = variant.number_of_suits as usize * variant.stacks_size as usize;
    for card_id in 0..num_unique {
        for _ in 0..ts.discard_pile.copies_of(card_id) {
            discard.push(json!(card_id_to_string(card_id)));
        }
    }

    let clue_tokens = ts.clue_token_bank.whole_clue_tokens_count();

    let hands: Vec<Value> = (0..num_players)
        .map(|p| {
            let slots: Vec<Value> = ts.hands[p]
                .cards()
                .iter()
                .map(|&deck_idx| {
                    let clue_touched = (ts.clue_touched_cards >> deck_idx) & 1 != 0;

                    if p != active {
                        // Emit the actual card identity (visible to the active player).
                        let id_str = tk.player(active).inferred_identities[deck_idx as usize]
                            .and_then(|m| m.known_card_id())
                            .map(card_id_to_string)
                            .unwrap_or_else(|| "x".to_string());

                        // Include the teammate's own convention knowledge when it carries
                        // more information than the plain card identity.
                        let inferred_str =
                            tk.player(p).inferred_identities[deck_idx as usize].and_then(|m| {
                                let trivial = m
                                    .known_card_id()
                                    .map(|c| card_id_to_string(c) == id_str)
                                    .unwrap_or(false);
                                if trivial { None } else { Some(mask_to_inferred_string(m.as_bits())) }
                            });

                        if let Some(inferred) = inferred_str {
                            let (pos, neg) = if clue_touched {
                                reconstruct_clues(
                                    ts.deck.get_global_empathy(deck_idx).as_bits(),
                                    variant,
                                )
                            } else {
                                (vec![], vec![])
                            };
                            json!({ "id": id_str, "positive": pos, "negative": neg, "inferred": inferred })
                        } else {
                            json!(id_str)
                        }
                    } else {
                        // Active player's own card: identity hidden, emit clue/convention info.
                        let deck_empathy =
                            ts.deck.get_global_empathy(deck_idx).as_bits();
                        let (pos, neg) = if clue_touched {
                            reconstruct_clues(deck_empathy, variant)
                        } else {
                            (vec![], vec![])
                        };
                        let inferred_str = tk.player(active).inferred_identities
                            [deck_idx as usize]
                            .map(|m| mask_to_inferred_string(m.as_bits()));

                        if pos.is_empty() && neg.is_empty() && inferred_str.is_none() {
                            json!("x")
                        } else {
                            let mut obj = serde_json::Map::new();
                            obj.insert("id".to_string(), json!("x"));
                            if !pos.is_empty() {
                                obj.insert("positive".to_string(), json!(pos));
                            }
                            if !neg.is_empty() {
                                obj.insert("negative".to_string(), json!(neg));
                            }
                            if let Some(inf) = inferred_str {
                                obj.insert("inferred".to_string(), json!(inf));
                            }
                            Value::Object(obj)
                        }
                    }
                })
                .collect();
            json!(slots)
        })
        .collect();

    json!({
        "scenario_description": "",
        "suits": suits,
        "playing_stacks": playing_stacks,
        "discard_pile": discard,
        "clue_tokens": clue_tokens,
        "strikes": ts.strike_tokens,
        "active_player": active,
        "hands": hands,
        "prior_actions": [],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
    use crate::engine::replay::reconstruct::{ReplayRunner, hand_size_for};
    use crate::external::hanablive::{Card, GameBuilder, GameOptions};
    use crate::game::state::table_state_json::{build_from_scenario, parse_scenario};
    use crate::game::static_game_data::StaticGameData;
    use crate::game::variant::test_variants::NO_VARIANT;

    fn ordered_deck() -> Vec<(usize, u8)> {
        let mut deck = Vec::new();
        for suit in 0..5usize {
            for rank in 1u8..=5 {
                let copies: u8 = match rank {
                    1 => 3,
                    2 | 3 | 4 => 2,
                    5 => 1,
                    _ => unreachable!(),
                };
                for _ in 0..copies {
                    deck.push((suit, rank));
                }
            }
        }
        deck
    }

    /// Build a ReplayRunner positioned after `num_actions` actions using the ordered deck.
    fn runner_after_actions<'a>(
        deck: &[(usize, u8)],
        action_tuples: Vec<(crate::external::hanablive::ActionType, usize, Option<usize>)>,
        conv: &'a HGroupConventionSet,
    ) -> ReplayRunner<'a> {
        let cards: Vec<Card> = deck
            .iter()
            .map(|&(suit_index, rank)| Card { suit_index, rank })
            .collect();

        let mut builder = GameBuilder::new(
            vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string(),
            ],
            cards,
        )
        .with_options({
            let mut opts = GameOptions::default();
            opts.variant = Some("No Variant".to_string());
            opts
        });

        for (action_type, target, value) in action_tuples {
            use crate::external::hanablive::ActionType;
            match action_type {
                ActionType::Play => builder.push_play(target),
                ActionType::Discard => builder.push_discard(target),
                ActionType::ColorClue => builder.push_color_clue(target, value.unwrap()),
                ActionType::RankClue => builder.push_rank_clue(target, value.unwrap()),
                ActionType::EndGame => {}
            }
        }

        let game = builder.finish();
        let mut runner = ReplayRunner::from_hanablive(&game, conv).unwrap();
        runner.step_to_turn(runner.total_turns()).unwrap();
        runner
    }

    #[test]
    fn roundtrip_preserves_table_state_after_play_and_discard() {
        use crate::external::hanablive::ActionType;

        let deck = ordered_deck();
        let conv = HGroupConventionSet::default();
        // Deck[0] = R1 → Alice plays it; deck[5] = Y1 → Bob discards it.
        let runner = runner_after_actions(
            &deck,
            vec![(ActionType::Play, 0, None), (ActionType::Discard, 5, None)],
            &conv,
        );

        let scenario_value = to_scenario_json(&runner.game);
        let json_str = serde_json::to_string(&scenario_value).unwrap();
        let scenario = parse_scenario(&json_str);
        let (loaded_ts, _sd) = build_from_scenario(&scenario, NO_VARIANT);

        assert_eq!(
            loaded_ts.playing_stacks, runner.game.table_state.playing_stacks,
            "playing stacks mismatch"
        );
        assert_eq!(
            loaded_ts.discard_pile, runner.game.table_state.discard_pile,
            "discard pile mismatch"
        );
        assert_eq!(
            loaded_ts.clue_token_bank.whole_clue_tokens_count(),
            runner
                .game
                .table_state
                .clue_token_bank
                .whole_clue_tokens_count(),
            "clue tokens mismatch"
        );
        assert_eq!(
            loaded_ts.strike_tokens, runner.game.table_state.strike_tokens,
            "strikes mismatch"
        );
        assert_eq!(
            loaded_ts.active_player_index, runner.game.table_state.active_player_index,
            "active player mismatch"
        );
        // Hands have different deck indices after play/draw (scenario normalises to 0..N),
        // so only compare hand sizes.
        for p in 0..3usize {
            assert_eq!(
                loaded_ts.hands[p].cards().len(),
                runner.game.table_state.hands[p].cards().len(),
                "player {p} hand size mismatch"
            );
        }
    }

    #[test]
    fn roundtrip_preserves_clue_touched_cards() {
        use crate::external::hanablive::ActionType;

        let deck = ordered_deck();
        let conv = HGroupConventionSet::default();
        // hand_size=5 for 3 players:
        //   Alice:   deck[0..4]  = R1 R1 R1 R2 R2 (oldest→newest)
        //   Bob:     deck[5..9]  = R3 R3 R4 R4 R5
        //   Charlie: deck[10..14] = Y1 Y1 Y1 Y2 Y2
        //
        // Alice gives a rank-3 clue to Bob (touches deck[5]=R3 and deck[6]=R3).
        let runner = runner_after_actions(&deck, vec![(ActionType::RankClue, 1, Some(3))], &conv);

        let scenario_value = to_scenario_json(&runner.game);
        let json_str = serde_json::to_string(&scenario_value).unwrap();
        let scenario = parse_scenario(&json_str);
        let (loaded_ts, _sd) = build_from_scenario(&scenario, NO_VARIANT);

        assert_eq!(
            loaded_ts.clue_touched_cards, runner.game.table_state.clue_touched_cards,
            "clue_touched_cards mismatch"
        );
    }

    #[test]
    fn card_id_to_string_matches_parse_card() {
        use crate::game::state::table_state_json::parse_card;
        for card_id in 0..25usize {
            let s = card_id_to_string(card_id);
            assert_eq!(
                parse_card(&s),
                card_id,
                "roundtrip failed for card_id {card_id}"
            );
        }
    }

    #[test]
    fn mask_to_inferred_string_roundtrip() {
        use crate::game::state::table_state_json::parse_empathy_mask;
        // G3 (id=12) and B3 (id=17)
        let mask: u64 = (1 << 12) | (1 << 17);
        let s = mask_to_inferred_string(mask);
        assert_eq!(parse_empathy_mask(&s), mask);
    }

    #[test]
    fn reconstruct_clues_rank_only() {
        // R3(2), Y3(7), G3(12), B3(17), P3(22) = rank-3 mask
        let rank3_mask: u64 = (1 << 2) | (1 << 7) | (1 << 12) | (1 << 17) | (1 << 22);
        let (pos, neg) = reconstruct_clues(rank3_mask, &NO_VARIANT);
        assert!(pos.contains(&"3".to_string()), "expected rank 3 positive");
        // Rank-3 mask overlaps every colour suit, so no colours are negative.
        assert!(
            !neg.iter()
                .any(|s| ["red", "yellow", "green", "blue", "purple"].contains(&s.as_str())),
            "no colour negatives expected for rank-3 mask"
        );
        // All other ranks are negative (they have no overlap with rank-3 mask).
        for rank in ["1", "2", "4", "5"] {
            assert!(
                neg.contains(&rank.to_string()),
                "expected rank {rank} negative"
            );
        }
    }

    #[test]
    fn reconstruct_clues_color_and_rank() {
        // B5 only (id=19)
        let b5_mask: u64 = 1 << 19;
        let (pos, neg) = reconstruct_clues(b5_mask, &NO_VARIANT);
        assert!(pos.contains(&"blue".to_string()), "expected blue positive");
        assert!(pos.contains(&"5".to_string()), "expected rank 5 positive");
        // 4 other colors negative, 4 other ranks negative
        assert_eq!(pos.len(), 2);
    }

    #[test]
    fn reconstruct_clues_reproduces_empathy_on_loading() {
        use crate::game::state::table_state_json::parse_clue_string;

        // {Y3, G3, B3}: rank-3 positive + red & purple negative
        let target_mask: u64 = (1 << 7) | (1 << 12) | (1 << 17);
        let (pos, neg) = reconstruct_clues(target_mask, &NO_VARIANT);

        // Simulate how team_knowledge_from_scenario applies them
        let all_cards_mask = NO_VARIANT.all_cards_mask();
        let mut empathy = all_cards_mask;
        for clue_str in &pos {
            let (ct, cv) = parse_clue_string(clue_str);
            let clue_mask = NO_VARIANT.empathy_by_clue(ct, cv as usize).as_bits();
            empathy &= clue_mask;
        }
        for clue_str in &neg {
            let (ct, cv) = parse_clue_string(clue_str);
            let clue_mask = NO_VARIANT.empathy_by_clue(ct, cv as usize).as_bits();
            empathy &= !clue_mask;
        }
        assert_eq!(
            empathy, target_mask,
            "clue reconstruction did not reproduce target empathy"
        );
    }

    #[test]
    fn to_scenario_json_active_player_is_correct() {
        let static_data = StaticGameData {
            number_of_players: 3,
            variant: NO_VARIANT,
        };
        let hand_size = hand_size_for(3);
        let conv = HGroupConventionSet::default();
        let deck_ids: Vec<usize> = ordered_deck()
            .iter()
            .map(|&(s, r)| s * 5 + (r as usize - 1))
            .collect();
        let runner = ReplayRunner::from_deck(deck_ids, static_data, &conv);

        let value = to_scenario_json(&runner.game);
        assert_eq!(value["active_player"], 0);
        assert_eq!(value["hands"].as_array().unwrap().len(), 3);
        // Alice's hand (player 0 = active): all slots should be "x" (own hand, no clues)
        let alice_hand = value["hands"][0].as_array().unwrap();
        assert_eq!(alice_hand.len(), hand_size);
        for slot in alice_hand {
            assert_eq!(
                slot.as_str(),
                Some("x"),
                "active player's unclued slot should be 'x'"
            );
        }
        // Bob's hand (player 1): should have concrete card strings
        let bob_hand = value["hands"][1].as_array().unwrap();
        assert_eq!(bob_hand.len(), hand_size);
        for slot in bob_hand {
            assert!(slot.is_string(), "teammate slot should be a card string");
            assert_ne!(slot.as_str(), Some("x"), "teammate slot should not be 'x'");
        }
    }
}
