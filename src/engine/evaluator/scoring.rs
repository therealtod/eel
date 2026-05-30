use crate::engine::convention::hgroup::h_group_core::{
    count_bad_touches, get_chop_index, is_potential_bad_touch,
};
use crate::engine::decision_tree::Score;
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

use super::default_evaluator::DefaultEvaluator;
use super::types::ScoreBreakdown;

impl super::Evaluator for DefaultEvaluator {
    fn score(
        &self,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> Score {
        self.score_breakdown(state, truth, phantom_plays).total
    }

    fn score_breakdown(
        &self,
        state: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
        phantom_plays: u8,
    ) -> ScoreBreakdown {
        let table_state = state.table_state();
        let static_data = state.static_data();
        let team_knowledge = state.team_knowledge();
        let game_score =
            self.score_weight * state.score(&static_data.variant, phantom_plays) as f64;
        let strikes = table_state.strike_tokens as usize;
        let strike_penalty = self.strike_penalties.get(strikes).copied().unwrap_or(0.0);
        if table_state.is_terminal(static_data) {
            return ScoreBreakdown {
                game_score,
                strike_penalty,
                pace: 0.0,
                efficiency_penalty: 0.0,
                critical_exposure_penalty: 0.0,
                lost_ceiling_penalty: 0.0,
                empathy_bonus: 0.0,
                clue_tokens: 0.0,
                known_playable: 0.0,
                team_empathy: 0.0,
                misinformation_penalty: 0.0,
                critical_exposure_delta: 0.0,
                tempo_cost: 0.0,
                total: game_score - strike_penalty,
            };
        }
        let pace = self.pace_weight
            * state
                .pace(phantom_plays)
                .clamp(-10, static_data.number_of_players as i32) as f64;
        let efficiency_penalty =
            self.efficiency_weight * f64::from(state.required_efficiency(phantom_plays));
        let critical_exposure_penalty = if self.critical_exposure_weight != 0.0 {
            self.critical_exposure_weight
                * DefaultEvaluator::critical_exposure_score(
                    table_state,
                    static_data,
                    team_knowledge,
                    truth,
                )
        } else {
            0.0
        };
        let theoretical_max =
            (static_data.variant.number_of_suits * static_data.variant.stacks_size) as f64;
        let lost_ceiling_penalty = self.lost_score_ceiling_weight
            * (theoretical_max - table_state.max_achievable_score(static_data) as f64);
        let empathy_bonus = if self.empathy_weight != 0.0 {
            self.empathy_weight
                * DefaultEvaluator::empathy_precision(table_state, static_data, team_knowledge)
        } else {
            0.0
        };
        let harmonic_value =
            DefaultEvaluator::harmonic(table_state.clue_token_bank.whole_clue_tokens_count());
        let demand_factor = if self.clue_demand_weight != 0.0 {
            1.0 + self.clue_demand_weight * DefaultEvaluator::clue_demand(table_state, static_data)
        } else {
            1.0
        };
        let clue_tokens = self.clue_token_weight * harmonic_value * demand_factor;
        let known_playable = if self.known_playable_weight != 0.0 {
            self.known_playable_weight
                * DefaultEvaluator::known_playable_in_hands(
                    table_state,
                    static_data,
                    team_knowledge,
                    truth,
                )
        } else {
            0.0
        };
        // team_empathy is no longer a leaf term — it accumulates through clue actions via
        // `team_empathy_delta_bonus`. The breakdown field is kept (zeroed here) so callers
        // that aggregate per-line bonuses can record what the clue deltas contributed.
        let team_empathy = 0.0;
        let misinformation_penalty = if self.misinformation_weight != 0.0 {
            self.misinformation_weight
                * DefaultEvaluator::misinformation_score(
                    static_data,
                    team_knowledge,
                    table_state,
                    truth,
                )
        } else {
            0.0
        };
        let total = game_score - strike_penalty + pace
            - efficiency_penalty
            - critical_exposure_penalty
            - lost_ceiling_penalty
            + empathy_bonus
            + clue_tokens
            + known_playable
            - misinformation_penalty;
        ScoreBreakdown {
            game_score,
            strike_penalty,
            pace,
            efficiency_penalty,
            critical_exposure_penalty,
            lost_ceiling_penalty,
            empathy_bonus,
            clue_tokens,
            known_playable,
            team_empathy,
            misinformation_penalty,
            critical_exposure_delta: 0.0,
            tempo_cost: 0.0,
            total,
        }
    }

    fn clue_precision_bonus(
        &self,
        touched: &[u8],
        receiver: usize,
        giver: usize,
        truth: &dyn PlayerPOV,
        static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> Score {
        let precision_bonus = if self.clue_precision_weight != 0.0 {
            touched
                .iter()
                .filter(|&&idx| {
                    let card_deck_index = idx as CardDeckIndex;
                    let combined = team_knowledge
                        .player(receiver)
                        .combined_possible_identities(
                            card_deck_index,
                            table_state,
                            &static_data.variant,
                        );
                    combined.is_exactly_known()
                })
                .count() as f64
                * self.clue_precision_weight
        } else {
            0.0
        };

        let bad_touch_count = count_bad_touches(touched, receiver, truth, table_state, static_data);

        let potential_bad_touch = self.potential_bad_touch_penalty != 0.0
            && is_potential_bad_touch(
                touched,
                giver,
                truth,
                table_state,
                static_data,
                team_knowledge,
            );

        precision_bonus
            - bad_touch_count as f64 * self.good_touch_penalty
            - if potential_bad_touch {
                self.potential_bad_touch_penalty
            } else {
                0.0
            }
    }

    fn signal_ignored_penalty(
        &self,
        action: &GameAction,
        actor: PlayerIndex,
        _static_data: &StaticGameData,
        team_knowledge: &TeamKnowledge,
        table_state: &TableState,
    ) -> Score {
        if self.signal_ignored_penalty_weight == 0.0 {
            return 0.0;
        }
        let pk = team_knowledge.player(actor);
        let played_card: Option<CardDeckIndex> = match action {
            GameAction::Play {
                card_deck_index, ..
            } => Some(*card_deck_index),
            _ => None,
        };
        let mut any_signal = false;
        let mut hand_mask = pk.own_hand;
        while hand_mask != 0 {
            let idx = hand_mask.trailing_zeros() as CardDeckIndex;
            hand_mask &= hand_mask - 1;
            if (table_state.clue_touched_cards >> idx) & 1 != 0 {
                continue;
            }
            if !pk.has_play_signal(idx) {
                continue;
            }
            any_signal = true;
            if played_card == Some(idx) {
                return 0.0;
            }
        }
        if any_signal {
            -self.signal_ignored_penalty_weight
        } else {
            0.0
        }
    }

    fn play_progress_bonus(
        &self,
        action: &GameAction,
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
        pre_phantom_plays: u8,
        post_phantom_plays: u8,
    ) -> Score {
        if self.play_progress_weight == 0.0 {
            return 0.0;
        }
        let GameAction::Play { .. } = action else {
            return 0.0;
        };
        let variant = &pre.static_data().variant;
        if post.score(variant, post_phantom_plays) > pre.score(variant, pre_phantom_plays) {
            self.play_progress_weight
        } else {
            0.0
        }
    }

    fn play_deferral_penalty(&self, elapsed_plies: usize) -> Score {
        self.play_deferral_weight * elapsed_plies as f64
    }

    fn discard_action_penalty(
        &self,
        action: &GameAction,
        actor: PlayerIndex,
        pre: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
    ) -> Score {
        let GameAction::Discard {
            card_deck_index, ..
        } = action
        else {
            return 0.0;
        };
        let table_state = pre.table_state();
        let static_data = pre.static_data();
        let bdr = if self.bottom_deck_risk_weight != 0.0 {
            self.bottom_deck_risk_weight
                * DefaultEvaluator::bottom_deck_risk_score(
                    *card_deck_index,
                    table_state,
                    static_data,
                    truth,
                )
        } else {
            0.0
        };
        let kp_penalty = if self.discard_while_known_playable_penalty != 0.0
            && DefaultEvaluator::actor_has_known_playable(
                actor,
                table_state,
                static_data,
                pre.team_knowledge(),
            ) {
            self.discard_while_known_playable_penalty
        } else {
            0.0
        };
        -(bdr + kp_penalty)
    }

    fn team_empathy_delta_bonus(
        &self,
        action: &GameAction,
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
    ) -> Score {
        if self.team_empathy_weight == 0.0 {
            return 0.0;
        }
        let GameAction::Clue { .. } = action else {
            return 0.0;
        };
        let static_data = pre.static_data();
        let pre_score = DefaultEvaluator::team_empathy_score(
            static_data,
            pre.team_knowledge(),
            pre.table_state(),
        );
        let post_score = DefaultEvaluator::team_empathy_score(
            static_data,
            post.team_knowledge(),
            post.table_state(),
        );
        self.team_empathy_weight * (post_score - pre_score)
    }

    fn critical_exposure_delta_bonus(
        &self,
        _action: &GameAction,
        actor: PlayerIndex,
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
        truth: &dyn PlayerPOV,
    ) -> Score {
        if self.critical_exposure_delta_weight == 0.0 {
            return 0.0;
        }
        let pre_table = pre.table_state();
        let post_table = post.table_state();
        let static_data = pre.static_data();
        let num_players = static_data.number_of_players as usize;

        let pre_critical_mask = DefaultEvaluator::critical_mask(pre_table, static_data);
        if pre_critical_mask == 0 {
            return 0.0;
        }

        let save_available = pre_table.clue_token_bank.whole_clue_tokens_count() > 0;
        let post_critical_mask = if save_available {
            DefaultEvaluator::critical_mask(post_table, static_data)
        } else {
            0
        };

        let playable_mask = pre_table.playable_cards(static_data);
        let actor_pk = pre.team_knowledge().player(actor);
        let actor_pov = LightweightPlayerPOV::new(
            actor,
            actor_pk,
            pre.team_knowledge(),
            pre_table,
            static_data,
        );
        let actor_known_playable =
            DefaultEvaluator::count_known_playable(&actor_pov, playable_mask);

        let mut total = 0.0f64;
        for t in 0..num_players {
            if t == actor {
                continue;
            }
            let pk = pre.team_knowledge().player(t);
            let pov =
                LightweightPlayerPOV::new(t, pk, pre.team_knowledge(), pre_table, static_data);
            let Some(chop_idx) = get_chop_index(t, &pov) else {
                continue;
            };
            let Some(chop_id) = truth.card_identity(chop_idx) else {
                continue;
            };
            if (1u64 << chop_id) & pre_critical_mask == 0 {
                continue;
            }
            let buffer = pre_table.hands[t]
                .cards()
                .iter()
                .filter(|&&idx| {
                    pov.is_known_trash(idx)
                        || DefaultEvaluator::card_known_playable(&pov, idx, playable_mask)
                })
                .count() as f64;
            let threat = 1.0 / (1.0 + 4.0 * buffer);

            let saved = (post_table.clue_touched_cards >> chop_idx) & 1 != 0;
            if saved {
                total += self.critical_exposure_delta_weight * threat;
            } else if save_available && (1u64 << chop_id) & post_critical_mask != 0 {
                let any_freer_intermediate = {
                    let mut p = (actor + 1) % num_players;
                    let mut found = false;
                    while p != t {
                        let int_pk = pre.team_knowledge().player(p);
                        let int_pov = LightweightPlayerPOV::new(
                            p,
                            int_pk,
                            pre.team_knowledge(),
                            pre_table,
                            static_data,
                        );
                        if DefaultEvaluator::count_known_playable(&int_pov, playable_mask)
                            < actor_known_playable
                        {
                            found = true;
                            break;
                        }
                        p = (p + 1) % num_players;
                    }
                    found
                };
                if !any_freer_intermediate {
                    total -= self.critical_exposure_delta_weight * threat;
                }
            }
        }
        total
    }

    fn tempo_cost(
        &self,
        action: &GameAction,
        pre: &KnowledgeAwareGameState,
        post: &KnowledgeAwareGameState,
        pre_phantom_plays: u8,
        post_phantom_plays: u8,
    ) -> Score {
        if self.tempo_cost_weight == 0.0 {
            return 0.0;
        }
        if matches!(action, GameAction::Discard { .. }) {
            return 0.0;
        }
        if matches!(action, GameAction::Play { .. }) {
            let variant = &pre.static_data().variant;
            // Successful play (real or phantom): score advances, no tempo cost.
            if post.score(variant, post_phantom_plays) > pre.score(variant, pre_phantom_plays) {
                return 0.0;
            }
            // Vanish play (`ResolvedPlay::Unknown`): card removed from hand without
            // resolving identity, no score advance, no strike — the leaf evaluator
            // sees no consequence either. Charging tempo here would create a
            // horizon-arbitrage asymmetry: a line that completes its productive
            // work early pads remaining plies with `LowLevelStall` priority-3
            // vanish-plays and pays full tempo per ply, while a line that stretches
            // productive clues+discards over those same plies pays much less.
            if post.table_state().strike_tokens == pre.table_state().strike_tokens {
                return 0.0;
            }
        }
        -self.tempo_cost_weight
    }

    fn upper_bound(
        &self,
        table_state: &TableState,
        static_data: &StaticGameData,
        depth: usize,
    ) -> Score {
        let max_game_score =
            self.score_weight * table_state.max_achievable_score(static_data) as f64;
        let min_strike_penalty = self
            .strike_penalties
            .get(table_state.strike_tokens as usize)
            .copied()
            .unwrap_or(0.0);
        let max_pace = self.pace_weight * static_data.number_of_players as f64;
        let num_players = static_data.number_of_players as usize;
        let total_cards: usize = table_state.hands[..num_players]
            .iter()
            .map(|h| h.cards().len())
            .sum();
        let max_demand = if self.clue_demand_weight > 0.0 {
            1.0 + self.clue_demand_weight * total_cards as f64
        } else {
            1.0
        };
        let max_clue_tokens = self.clue_token_weight
            * DefaultEvaluator::harmonic(crate::game::MAX_CLUE_TOKEN_COUNT)
            * max_demand;
        let max_known_playable = self.known_playable_weight * total_cards as f64;
        let max_leaf =
            max_game_score - min_strike_penalty + max_pace + max_clue_tokens + max_known_playable;
        let max_immediate_per_ply = self.play_progress_weight
            + self.clue_precision_weight * total_cards as f64
            + self.team_empathy_weight * total_cards as f64
            + self.critical_exposure_delta_weight
                * (static_data.number_of_players as f64 - 1.0).max(0.0);
        max_leaf + max_immediate_per_ply * depth as f64
    }
}
