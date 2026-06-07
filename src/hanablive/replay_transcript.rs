//! Offline replay of a recorded liveplay transcript.
//!
//! Feeds a transcript captured by [`LiveplayRecorder`](crate::hanablive::liveplay_recorder) — one
//! raw `label payload` frame per line — through the **exact** live code path: each line is parsed
//! with [`InboundWebsocketMessage::parse`] and applied to a real [`BotState`], with a
//! [`NullSink`] standing in for the websocket. This reproduces live behavior faithfully, including
//! the two things the full-deck `replay.json` reconstruction cannot: the hidden own-hand draws
//! (our cards arrive as `suit/rank = -1`) and the server's exact clue `list` / `Turn` ordering.
//!
//! Use it to debug a divergence between live and selfplay: replay the transcript to the turn where
//! the bot misbehaved and inspect the recommendation (and the [`BotState`]'s knowledge) from the
//! same state the live bot actually decided from.

use std::fmt;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::hgroup::h_group_convention_set::{HGroupConventionSet, HGroupLevel};
use crate::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use crate::game::action::game_action::GameAction;
use crate::game::variant::test_variants::NO_VARIANT;
use crate::hanablive::bot::bot_state::BotState;
use crate::hanablive::client::outbound_sink::NullSink;
use crate::hanablive::dto::payload::game::Game;
use crate::hanablive::dto::payload::game_action::GameActionDto;
use crate::hanablive::inbound_websocket_message::InboundWebsocketMessage;

/// Why a transcript replay could not produce a recommendation.
#[derive(Debug)]
pub enum TranscriptError {
    /// No `init` frame was found, so the game could never be set up.
    NoInit,
    /// The transcript ended before `current_turn` reached `target`. `last` is the highest turn
    /// actually reached (`None` if the game never entered the `Playing` state — e.g. an
    /// unsupported variant made `init` revert to idle).
    NeverReachedTurn { target: usize, last: Option<usize> },
}

impl fmt::Display for TranscriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranscriptError::NoInit => write!(f, "transcript contains no `init` frame"),
            TranscriptError::NeverReachedTurn { target, last } => write!(
                f,
                "transcript ended before reaching turn {target} (highest turn reached: {last:?})"
            ),
        }
    }
}

impl std::error::Error for TranscriptError {}

/// The `BotState` reached after replaying a transcript to a target turn, paired with the
/// convention set used to interpret it. Inspect [`Self::state`] for the engine's knowledge, or
/// call [`Self::engine_recommendation`] for the action the bot would take at this turn.
pub struct TranscriptReplay {
    pub state: BotState,
    pub convention_set: HGroupConventionSet,
}

impl TranscriptReplay {
    /// The action the engine recommends for the active player at the reached turn — the same
    /// computation the live bot's `take_turn` performs.
    ///
    /// # Panics
    ///
    /// Panics if the replay did not end in the `Playing` state (it always does when constructed
    /// via [`replay_to_turn`], which only returns `Ok` once `Playing` is reached).
    #[must_use]
    pub fn engine_recommendation(&self) -> GameAction {
        let BotState::Playing { game, .. } = &self.state else {
            panic!("transcript replay did not reach the Playing state");
        };
        let active = game.table_state.active_player_index;
        let pov = game.player_pov(active);
        let strategy = TreeActionSelectionStrategy::default();
        strategy.select_active_player_action(&pov, &self.convention_set)
    }
}

/// Replay `transcript` through the live `BotState` until the engine reaches `target_turn`, then
/// return the resulting state for inspection.
///
/// `target_turn` is 1-based and matches the `ReplayRunner` harness used by the replay-regression
/// tests: turn 1 is the opening decision (no action applied yet), turn N is reached once N-1
/// actions have been applied. The same turn number therefore selects the same decision point in
/// both harnesses.
///
/// Frames before `init` (lobby chatter, table list, `tableStart`) are ignored; the game is set up
/// from the `init` frame onward exactly as live play does. Replay stops as soon as the target turn
/// is reached — i.e. *before* the action recorded for that turn is applied — so the returned state
/// is the decision point, not its aftermath.
///
/// # Errors
///
/// Returns [`TranscriptError::NoInit`] if there is no `init` frame, or
/// [`TranscriptError::NeverReachedTurn`] if the transcript ends first.
pub fn replay_to_turn(
    transcript: &str,
    hgroup_level: HGroupLevel,
    target_turn: usize,
) -> Result<TranscriptReplay, TranscriptError> {
    // The handshake methods are async only to await the (here no-op) sink, so drive them on a
    // minimal current-thread runtime and keep this entry point synchronous, matching the other
    // replay harnesses.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("failed to build current-thread runtime");
    runtime.block_on(replay_to_turn_async(transcript, hgroup_level, target_turn))
}

/// Convenience wrapper over [`replay_to_turn`] returning just the engine's recommended action at
/// `target_turn`.
///
/// # Errors
///
/// See [`replay_to_turn`].
pub fn engine_action_at_turn(
    transcript: &str,
    hgroup_level: HGroupLevel,
    target_turn: usize,
) -> Result<GameAction, TranscriptError> {
    Ok(replay_to_turn(transcript, hgroup_level, target_turn)?.engine_recommendation())
}

async fn replay_to_turn_async(
    transcript: &str,
    hgroup_level: HGroupLevel,
    target_turn: usize,
) -> Result<TranscriptReplay, TranscriptError> {
    let messages: Vec<InboundWebsocketMessage> = transcript
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(InboundWebsocketMessage::parse)
        .collect();

    // The game's table_id comes from the `init` frame; `on_init_received` expects the state to
    // already be `WaitingForInit` for that table, so seed it before replaying.
    let table_id = messages
        .iter()
        .find_map(|m| match m {
            InboundWebsocketMessage::Init(data) => Some(data.table_id),
            _ => None,
        })
        .ok_or(TranscriptError::NoInit)?;

    let convention_set = HGroupConventionSet::for_level(hgroup_level, &NO_VARIANT);
    let mut sink = NullSink;
    let mut state = BotState::WaitingForInit {
        tables: Vec::new(),
        table_id,
    };

    // Count turn-advancing actions (play/discard/clue) applied so far. The decision point for
    // turn `T` is reached once `T - 1` such actions — and all their trailing draw/turn frames —
    // have been applied, i.e. right *before* the `T`-th turn action's frame. Stopping there (not
    // merely when `current_turn` ticks over on the action frame) matches `ReplayRunner`, which
    // applies each action together with its replacement draw before yielding the position, and
    // mirrors how the live bot only acts once the previous actor's draw/turn frames have arrived.
    //
    // This assumes turn actions arrive as individual `gameAction` frames (true for a bot that
    // played from game start, and for [`synthesize_transcript`]); a batched `gameActionList`
    // carrying mid-history turn actions is applied wholesale and not split at the target.
    let mut turns_applied: usize = 0;
    let mut reached_playing = false;
    let stop_after = target_turn.saturating_sub(1);

    for message in messages {
        match message {
            InboundWebsocketMessage::Init(init_data) => {
                state.on_init_received(init_data, &mut sink).await;
            }
            InboundWebsocketMessage::CardIdentities(payload) => {
                let _ = state.on_card_identities_received(payload, &mut sink).await;
            }
            InboundWebsocketMessage::GameActionList(payload) => {
                let _ = state
                    .on_game_action_list_received(&payload, &convention_set, &mut sink)
                    .await;
            }
            InboundWebsocketMessage::GameAction(payload) => {
                if is_turn_action(&payload.action) {
                    if reached_playing && turns_applied == stop_after {
                        // Reached the decision point: stop before applying the target action.
                        return Ok(TranscriptReplay {
                            state,
                            convention_set,
                        });
                    }
                    let _ = state.apply_game_action(&payload, &convention_set);
                    turns_applied += 1;
                } else {
                    // Draw / Turn / Status / Strike / GameOver: state-tracking only, no turn change.
                    let _ = state.apply_game_action(&payload, &convention_set);
                }
            }
            // Lobby/handshake and other frames carry no game-state transition here.
            _ => {}
        }

        reached_playing |= matches!(state, BotState::Playing { .. });
    }

    // The transcript ended. If we applied at least `target_turn - 1` actions, the target turn's
    // decision point was reached (its action simply isn't in the transcript).
    if reached_playing && turns_applied >= stop_after {
        return Ok(TranscriptReplay {
            state,
            convention_set,
        });
    }

    Err(TranscriptError::NeverReachedTurn {
        target: target_turn,
        last: reached_playing.then_some(turns_applied + 1),
    })
}

/// Whether a network action advances the turn counter (a player's actual move), as opposed to the
/// bookkeeping frames (draw, turn, status, strike, game-over) the server interleaves.
fn is_turn_action(action: &GameActionDto) -> bool {
    matches!(
        action,
        GameActionDto::Play { .. } | GameActionDto::Discard { .. } | GameActionDto::Clue { .. }
    )
}

/// Synthesize a live-equivalent transcript from a hanab.live `Game` export, with
/// `our_player_index`'s own hand hidden (drawn as `suit/rank = -1`, exactly as the live server
/// sends our own cards).
///
/// The emitted frames mirror real play: an `init`, a `gameActionList` for the initial deal, then
/// per recorded action a `gameAction` (play/discard/clue) followed by the actor's replacement
/// `draw` and a `turn` hand-off. Plays/discards reveal their card (it becomes public); the hidden
/// seat's *replacement draws* stay `-1`. The starting player is assumed to be player 0, matching
/// the [`ReplayRunner`](crate::engine::replay::reconstruct::ReplayRunner) convention.
///
/// The export carries no touched-card `list` for clues, so it is recomputed from the deck (the
/// same way `ReplayRunner` does). A transcript built this way therefore exercises the live
/// `BotState` code path on identical information to `ReplayRunner` — use it to A/B the two
/// reconstructions for code-path equivalence. (To reproduce a *real* divergence that depends on
/// server-only data, e.g. clue `list` ordering, use a transcript captured by the live recorder
/// instead.)
#[must_use]
#[allow(clippy::cast_possible_wrap)]
pub fn synthesize_transcript(game: &Game, our_player_index: usize) -> String {
    use crate::engine::replay::reconstruct::hand_size_for;
    use crate::hanablive::action::ActionType;

    const TABLE_ID: u32 = 1;

    let num_players = game.players.len();
    let hand_size = hand_size_for(num_players as u8);
    let deck = &game.deck;
    let deck_total = deck.len();

    // Build a `draw` action value, hiding identity for the hidden seat.
    let draw_value = |player: usize, order: usize, hidden: bool| {
        let (suit, rank) = if hidden {
            (-1_i64, -1_i64)
        } else {
            (deck[order].suit_index as i64, i64::from(deck[order].rank))
        };
        serde_json::json!({
            "type": "draw", "playerIndex": player, "order": order,
            "suitIndex": suit, "rank": rank,
        })
    };
    let frame = |action: serde_json::Value| -> String {
        format!(
            "gameAction {}",
            serde_json::json!({ "tableID": TABLE_ID, "action": action })
        )
    };

    let mut lines: Vec<String> = Vec::new();

    let init = serde_json::json!({
        "tableID": TABLE_ID,
        "playerNames": game.players,
        "ourPlayerIndex": our_player_index,
        "spectating": false, "shadowing": false, "replay": false,
        "databaseID": -1, "hasCustomSeed": false, "seed": "synth",
        "datetimeStarted": "2024-01-01T00:00:00Z", "datetimeFinished": "2024-01-01T00:00:00Z",
        "options": {
            "maxPlayers": null, "numPlayers": num_players, "startingPlayer": 0,
            "variantID": 0, "variantName": "No Variant", "tableName": null,
            "timed": false, "timeBase": 0, "timePerTurn": 0, "speedrun": false,
            "cardCycle": false, "deckPlays": false, "emptyClues": false,
            "oneExtraCard": false, "oneLessCard": false, "allOrNothing": false,
            "detrimentalCharacters": false
        },
        "characterAssignments": [], "characterMetadata": [],
        "sharedReplay": false, "sharedReplayLeader": "", "sharedReplaySegment": 0,
        "sharedReplayEffMod": 0, "paused": false, "pausePlayerIndex": -1, "pauseQueued": false
    });
    lines.push(format!("init {init}"));

    // Initial deal, tracking each player's hand (deck orders) so clue targets can be recomputed.
    let mut hands: Vec<Vec<usize>> = vec![Vec::new(); num_players];
    let mut deal_list: Vec<serde_json::Value> = Vec::new();
    for (player, hand) in hands.iter_mut().enumerate() {
        for slot in 0..hand_size {
            let order = player * hand_size + slot;
            hand.push(order);
            deal_list.push(draw_value(player, order, player == our_player_index));
        }
    }
    lines.push(format!(
        "gameActionList {}",
        serde_json::json!({ "tableID": TABLE_ID, "list": deal_list })
    ));

    let mut active = 0usize;
    let mut next_order = num_players * hand_size;

    for action in &game.actions {
        match action.action_type {
            ActionType::EndGame => continue,
            ActionType::Play | ActionType::Discard => {
                let order = action.target;
                let is_play = matches!(action.action_type, ActionType::Play);
                let mut act = serde_json::json!({
                    "type": if is_play { "play" } else { "discard" },
                    "playerIndex": active, "order": order,
                    "suitIndex": deck[order].suit_index as i64,
                    "rank": i64::from(deck[order].rank),
                });
                if !is_play {
                    act["failed"] = serde_json::Value::Bool(false);
                }
                lines.push(frame(act));
                hands[active].retain(|&o| o != order);
                // Replacement draw (only while the deck has cards left).
                if next_order < deck_total {
                    hands[active].push(next_order);
                    lines.push(frame(draw_value(
                        active,
                        next_order,
                        active == our_player_index,
                    )));
                    next_order += 1;
                }
            }
            ActionType::ColorClue | ActionType::RankClue => {
                let receiver = action.target;
                let value = action.value.unwrap_or(0);
                let is_color = matches!(action.action_type, ActionType::ColorClue);
                let list: Vec<usize> = hands[receiver]
                    .iter()
                    .copied()
                    .filter(|&o| {
                        if is_color {
                            deck[o].suit_index == value
                        } else {
                            deck[o].rank as usize == value
                        }
                    })
                    .collect();
                lines.push(frame(serde_json::json!({
                    "type": "clue",
                    "clue": { "type": if is_color { 0 } else { 1 }, "value": value },
                    "giver": active, "list": list, "target": receiver, "turn": 0,
                })));
            }
        }

        active = (active + 1) % num_players;
        lines.push(frame(serde_json::json!({
            "type": "turn", "num": 0, "currentPlayerIndex": active,
        })));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{TranscriptError, engine_action_at_turn, replay_to_turn};
    use crate::engine::convention::hgroup::h_group_convention_set::HGroupLevel;
    use crate::game::action::game_action::GameAction;
    use crate::hanablive::bot::bot_state::BotState;

    /// A 2-player "No Variant" `init` frame; we are player 0 and start.
    const INIT: &str = r#"init {"tableID":1,"playerNames":["us","them"],"ourPlayerIndex":0,"spectating":false,"shadowing":false,"replay":false,"databaseID":-1,"hasCustomSeed":false,"seed":"s","datetimeStarted":"2024-01-01T00:00:00Z","datetimeFinished":"2024-01-01T00:00:00Z","options":{"maxPlayers":null,"numPlayers":2,"startingPlayer":0,"variantID":0,"variantName":"No Variant","tableName":null,"timed":false,"timeBase":0,"timePerTurn":0,"speedrun":false,"cardCycle":false,"deckPlays":false,"emptyClues":false,"oneExtraCard":false,"oneLessCard":false,"allOrNothing":false,"detrimentalCharacters":false},"characterAssignments":[],"characterMetadata":[],"sharedReplay":false,"sharedReplayLeader":"","sharedReplaySegment":0,"sharedReplayEffMod":0,"paused":false,"pausePlayerIndex":-1,"pauseQueued":false}"#;

    /// The initial deal: our 5 cards arrive hidden (`-1/-1`), the opponent's 5 are visible
    /// (orders 5–9 = R1 R1 R1 R2 R2). This is exactly how the live server frames a fresh game.
    const DEAL: &str = r#"gameActionList {"tableID":1,"list":[{"type":"draw","playerIndex":0,"order":0,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":0,"order":1,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":0,"order":2,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":0,"order":3,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":0,"order":4,"suitIndex":-1,"rank":-1},{"type":"draw","playerIndex":1,"order":5,"suitIndex":0,"rank":1},{"type":"draw","playerIndex":1,"order":6,"suitIndex":0,"rank":1},{"type":"draw","playerIndex":1,"order":7,"suitIndex":0,"rank":1},{"type":"draw","playerIndex":1,"order":8,"suitIndex":0,"rank":2},{"type":"draw","playerIndex":1,"order":9,"suitIndex":0,"rank":2}]}"#;

    #[test]
    fn reaches_first_turn_with_hidden_own_hand() {
        let transcript = format!("{INIT}\n{DEAL}\n");
        let replay = replay_to_turn(&transcript, HGroupLevel::One, 1).expect("reach turn 1");

        let BotState::Playing {
            our_player_index,
            game,
            ..
        } = &replay.state
        else {
            panic!("expected Playing state");
        };
        assert_eq!(*our_player_index, 0);
        assert_eq!(
            game.current_turn(),
            0,
            "0-based: no action applied yet (turn 1)"
        );
        assert_eq!(game.table_state.active_player_index, 0, "we start");
        // Our own hand stays hidden from teammates — the divergence the full-deck reconstruction
        // can't reproduce. Player 1 must not see our dealt cards (orders 0..5).
        let our_hand_bits: u64 = (0..5).map(|i| 1u64 << i).sum();
        assert_eq!(
            game.team_knowledge.player(1).visible_cards & our_hand_bits,
            0,
            "opponent must not see our own hand in the live-faithful reconstruction"
        );

        // The engine produces a concrete (non-draw) action for the active player.
        assert!(!matches!(
            replay.engine_recommendation(),
            GameAction::Draw { .. }
        ));
    }

    #[test]
    fn applies_a_clue_and_advances_to_opponent_turn() {
        // We clue red to the opponent (touching their three R1s at orders 5,6,7), which advances
        // the turn to the opponent.
        let clue = r#"gameAction {"tableID":1,"action":{"type":"clue","clue":{"type":0,"value":0},"giver":0,"list":[5,6,7],"target":1,"turn":0}}"#;
        let transcript = format!("{INIT}\n{DEAL}\n{clue}\n");

        let replay = replay_to_turn(&transcript, HGroupLevel::One, 2).expect("reach turn 2");
        let BotState::Playing { game, .. } = &replay.state else {
            panic!("expected Playing state");
        };
        assert_eq!(
            game.current_turn(),
            1,
            "0-based: one action applied (turn 2)"
        );
        assert_eq!(
            game.table_state.active_player_index, 1,
            "after our clue it is the opponent's turn"
        );
    }

    #[test]
    fn missing_init_is_an_error() {
        let err = engine_action_at_turn(DEAL, HGroupLevel::One, 1).unwrap_err();
        assert!(matches!(err, TranscriptError::NoInit));
    }

    #[test]
    fn running_past_the_end_reports_last_turn() {
        let transcript = format!("{INIT}\n{DEAL}\n");
        let err = replay_to_turn(&transcript, HGroupLevel::One, 99)
            .err()
            .expect("should not reach turn 99");
        assert!(matches!(
            err,
            TranscriptError::NeverReachedTurn {
                target: 99,
                last: Some(1)
            }
        ));
    }
}
