use smallvec::SmallVec;

use crate::engine::action_selection_strategy::ActionSelectionStrategy;
use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::engine::play_resolver::GroundTruthResolver;
use crate::engine::replay::reconstruct::hand_size_for;
use crate::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use crate::game::MAX_HAND_SIZE;
use crate::game::action::game_action::GameAction;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::clue_type::ClueType;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::test_variants::NO_VARIANT;
use crate::hanablive::TableId;
use crate::hanablive::action::ActionType;
use crate::hanablive::client::outbound_sink::OutboundSink;
use crate::hanablive::dto::payload::card_identity::CardIdentitiesPayload;
use crate::hanablive::dto::payload::game_action::{
    GameActionDto, GameActionListPayload, GameActionPayload,
};
use crate::hanablive::dto::payload::game_init_data::GameInitData;
use crate::hanablive::dto::payload::table::Table;
use crate::hanablive::outbound_websocket_message::OutboundWebsocketMessage;

/// The only variant the engine currently understands. Tables using any other
/// variant are declined rather than played with incorrect rules — playing garbage
/// on a public server is worse than not playing.
const SUPPORTED_VARIANT: &str = "No Variant";

/// Whether the engine can play the given variant. See [`SUPPORTED_VARIANT`].
fn is_supported_variant(variant_name: &str) -> bool {
    variant_name == SUPPORTED_VARIANT
}

/// Record a server-revealed card identity into `actual_deck` at `order`, growing the deck if
/// needed. A `suit_index`/`rank` of `-1` means the identity is still hidden (our own draws), so
/// nothing is recorded. Used by `Draw`, `Play`, and `Discard` actions, all of which carry the
/// identity once the server reveals it.
fn record_revealed_identity(
    actual_deck: &mut Vec<VariantCardId>,
    static_data: &StaticGameData,
    order: usize,
    suit_index: i32,
    rank: i32,
) {
    if suit_index < 0 || rank < 0 {
        return;
    }
    let stacks_size = static_data.variant.stacks_size as usize;
    let card_id = suit_index as usize * stacks_size + (rank as usize - 1);
    if order >= actual_deck.len() {
        actual_deck.resize(order + 1, 0);
    }
    actual_deck[order] = card_id;
}

/// The result of applying one or more network actions to the bot's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnOutcome {
    /// It is now our turn; the bot should select and send an action.
    OurTurn,
    /// Nothing for the bot to do; keep waiting for server messages.
    Waiting,
    /// The game has ended (a `gameOver` action was observed). The bot should
    /// leave the table and return to idle.
    GameOver,
}

/// The bot's lifecycle state.
///
/// **Single-table limitation (deliberate, #8).** This state machine tracks at most one
/// joined/played table at a time: a single `table_id`, one `our_player_index`, and one
/// `game`. `find_table_for_user`, the `on_table_*` handlers, and the `Playing` variant all
/// assume this. Supporting spectating or concurrent tables would require keying these by
/// `table_id` (e.g. a map of per-table sub-states). The `TableSpectate`/`SpectateTable`
/// paths are stubbed and currently unreachable for the same reason. This is a chosen first
/// scope, not a latent bug — revisit before enabling multi-table or spectator play.
pub enum BotState {
    Idle {
        tables: Vec<Table>,
    },
    TableJoined {
        tables: Vec<Table>,
        table_id: TableId,
    },
    WaitingForInit {
        tables: Vec<Table>,
        table_id: TableId,
    },
    WaitingForGameActionList {
        tables: Vec<Table>,
        table_id: TableId,
        init_data: GameInitData,
        static_data: StaticGameData,
    },
    Playing {
        tables: Vec<Table>,
        table_id: TableId,
        our_player_index: usize,
        game: KnowledgeAwareGameState,
        actual_deck: Vec<VariantCardId>,
        static_data: StaticGameData,
    },
}

impl BotState {
    pub fn initial_state() -> Self {
        BotState::Idle { tables: vec![] }
    }

    fn tables_mut(&mut self) -> &mut Vec<Table> {
        match self {
            BotState::Idle { tables }
            | BotState::TableJoined { tables, .. }
            | BotState::WaitingForInit { tables, .. }
            | BotState::WaitingForGameActionList { tables, .. }
            | BotState::Playing { tables, .. } => tables,
        }
    }

    pub fn on_table_list_received(&mut self, tables: Vec<Table>) {
        *self.tables_mut() = tables;
    }

    pub fn on_table_added(&mut self, table: Table) {
        self.tables_mut().push(table);
    }

    pub fn on_table_removed(&mut self, table_id: TableId) {
        self.tables_mut().retain(|t| t.id != table_id);
    }

    pub fn find_table_for_user(&mut self, username: &str) -> Option<TableId> {
        self.tables_mut()
            .iter()
            .find(|table| table.players.contains(&String::from(username)))
            .map(|table| table.id)
    }

    pub async fn on_table_join_request(
        &mut self,
        table_id: TableId,
        password: Option<String>,
        websocket_client: &mut dyn OutboundSink,
    ) {
        let BotState::Idle { tables } = self else {
            tracing::info!("Cannot join a table in the current state");
            return;
        };
        if let Some(table) = tables.iter().find(|t| t.id == table_id) {
            if !is_supported_variant(&table.variant) {
                tracing::warn!(
                    "Refusing to join table {table_id}: unsupported variant '{}'",
                    table.variant
                );
                return;
            }
        }
        let msg = OutboundWebsocketMessage::TableJoin(table_id, password);
        if let Err(e) = websocket_client.send(&msg).await {
            tracing::error!("Failed to send table join request: {}", e);
            return;
        }
        let tables = std::mem::take(tables);
        *self = BotState::TableJoined { tables, table_id };
    }

    pub async fn on_table_leave_request(&mut self, websocket_client: &mut dyn OutboundSink) {
        let BotState::TableJoined { tables, table_id } = self else {
            tracing::info!("Cannot leave a table in the current state");
            return;
        };
        let msg = OutboundWebsocketMessage::TableLeave(*table_id);
        if let Err(e) = websocket_client.send(&msg).await {
            tracing::error!("Failed to send table leave request: {}", e);
            return;
        }
        let tables = std::mem::take(tables);
        *self = BotState::Idle { tables };
    }

    /// Handle a `tableStart` message. If the table ID matches, sends `getGameInfo1` and
    /// transitions to `WaitingForInit`.
    pub async fn on_table_start(
        &mut self,
        incoming_table_id: TableId,
        websocket_client: &mut dyn OutboundSink,
    ) {
        let BotState::TableJoined { table_id, .. } = self else {
            return;
        };
        if *table_id != incoming_table_id {
            return;
        }
        let table_id = *table_id;

        let msg = OutboundWebsocketMessage::GetGameInfo1(table_id);
        if let Err(e) = websocket_client.send(&msg).await {
            tracing::error!("Failed to send GetGameInfo1: {e}");
            return;
        }
        tracing::info!("Table {incoming_table_id} started, waiting for init");
        let tables = std::mem::take(self.tables_mut());
        *self = BotState::WaitingForInit { tables, table_id };
    }

    /// Handle an `init` message. Sends `getGameInfo2`, builds `StaticGameData`, and
    /// transitions to `WaitingForCardIdentities`.
    ///
    /// # Panics
    ///
    /// Panics if the server reports a player count that does not fit in a `u8` (never the case
    /// for real Hanabi games, which top out at 6 players).
    pub async fn on_init_received(
        &mut self,
        init_data: GameInitData,
        websocket_client: &mut dyn OutboundSink,
    ) {
        let BotState::WaitingForInit { table_id, .. } = self else {
            tracing::warn!("Received init in unexpected state, ignoring");
            return;
        };
        let table_id = *table_id;

        // Safety net: the join-time guard should already have prevented this, but
        // never play a variant we don't understand. Leave the table instead.
        if !is_supported_variant(&init_data.options.variant_name) {
            tracing::error!(
                "Game started with unsupported variant '{}'; leaving table {table_id}",
                init_data.options.variant_name
            );
            let unattend = OutboundWebsocketMessage::TableUnattend(table_id);
            if let Err(e) = websocket_client.send(&unattend).await {
                tracing::error!("Failed to send tableUnattend for unsupported variant: {e}");
            }
            let tables = std::mem::take(self.tables_mut());
            *self = BotState::Idle { tables };
            return;
        }

        let number_of_players =
            u8::try_from(init_data.options.num_players).expect("player count fits in u8");
        let static_data = StaticGameData {
            number_of_players,
            variant: NO_VARIANT,
        };

        tracing::info!(
            "Game initialized: {} players, seed={}",
            static_data.number_of_players,
            init_data.seed
        );

        let tables = std::mem::take(self.tables_mut());
        let msg = OutboundWebsocketMessage::GetGameInfo2(table_id);
        if let Err(e) = websocket_client.send(&msg).await {
            tracing::error!("Failed to send GetGameInfo2: {e}");
            return;
        }

        *self = BotState::WaitingForGameActionList {
            tables,
            table_id,
            init_data,
            static_data,
        };
    }

    /// Handle a `gameActionList` message arriving in `WaitingForGameActionList` state.
    ///
    /// The server sends a `gameActionList` in response to `getGameInfo2`. The list begins with
    /// one `Draw` action per dealt card (all players' initial hands). Cards dealt to us have
    /// `suit_index == -1` and `rank == -1`; all others carry the real identity. After consuming
    /// the draw actions, the state transitions to `Playing` (sending `loaded`) and any remaining
    /// actions (e.g. clues/plays already in the history) are processed normally.
    ///
    /// If the bot is already `Playing`, the call falls through to [`Self::apply_action_list`].
    ///
    /// # Panics
    ///
    /// Panics if a deck index does not fit in a `u8` (never the case for real Hanabi games).
    pub async fn on_game_action_list_received(
        &mut self,
        payload: &GameActionListPayload,
        convention_set: &HGroupConventionSet,
        websocket_client: &mut dyn OutboundSink,
    ) -> TurnOutcome {
        if !matches!(self, BotState::WaitingForGameActionList { .. }) {
            return self.apply_action_list(payload, convention_set);
        }

        let BotState::WaitingForGameActionList {
            tables,
            table_id,
            init_data,
            static_data,
        } = std::mem::replace(self, BotState::Idle { tables: Vec::new() })
        else {
            unreachable!("matched WaitingForGameActionList above")
        };

        let stacks_size = static_data.variant.stacks_size as usize;

        // Build the actual_deck from Draw actions where the identity is known.
        // We allocate enough capacity up front but fill lazily; unknown slots are left as 0
        // and are never used for our own cards (we never peek at our own cards in actual_deck).
        let draw_count = payload
            .list
            .iter()
            .filter(|a| matches!(a, GameActionDto::Draw { .. }))
            .count();
        let mut actual_deck: Vec<VariantCardId> = vec![0; draw_count];

        let mut game = KnowledgeAwareGameState::new(static_data.clone());

        // Process draws in order, setting active_player as each draw dictates.
        for action in &payload.list {
            let GameActionDto::Draw {
                player_index,
                order,
                suit_index,
                rank,
            } = action
            else {
                break; // initial draws are always first; stop at the first non-draw
            };

            let deck_idx = CardDeckIndex::try_from(*order).expect("deck index fits in u8");

            game.table_state.active_player_index = *player_index;

            if *suit_index >= 0 && *rank >= 0 {
                // Known card — visible to us because it belongs to another player.
                let card_id = *suit_index as usize * stacks_size + (*rank as usize - 1);
                actual_deck[*order] = card_id;
                game.update_with_draw_action_of_specific_card(*player_index, deck_idx, card_id);
            } else {
                // Unknown card — dealt to our own hand; identity hidden.
                game.update_with_draw_action(*player_index, deck_idx);
                game.next_deck_index = deck_idx + 1;
            }
        }

        game.table_state.active_player_index = init_data.options.starting_player;
        // next_deck_index must be past all dealt cards.
        game.next_deck_index = draw_count as u8;

        let our_player_index = init_data.our_player_index;
        let table_id_val = table_id;

        if let Err(e) = websocket_client
            .send(&OutboundWebsocketMessage::Loaded(table_id_val))
            .await
        {
            tracing::error!("Failed to send Loaded: {e}");
        }

        tracing::info!("Game ready: we are player {our_player_index}");

        *self = BotState::Playing {
            tables,
            table_id: table_id_val,
            our_player_index,
            game,
            actual_deck,
            static_data,
        };

        // Apply any non-draw actions that follow (history replay).
        let post_draw_actions: Vec<_> = payload
            .list
            .iter()
            .skip_while(|a| matches!(a, GameActionDto::Draw { .. }))
            .cloned()
            .collect();

        if post_draw_actions.is_empty() {
            return self.turn_outcome(false);
        }

        let replay_payload = GameActionListPayload {
            table_id: table_id_val,
            list: post_draw_actions,
        };
        self.apply_action_list(&replay_payload, convention_set)
    }

    /// Handle a `cardIdentities` message. Deals initial hands, sends `loaded`, and
    /// transitions to `Playing`.
    ///
    /// Returns the resulting [`TurnOutcome`] so the caller can act when we are the starting
    /// player: dealing leaves `active_player_index` at the server's `starting_player`, and if
    /// that is us nothing else will arrive to prompt us (it is our turn, so no opponent action
    /// is coming). The caller must therefore drive `take_turn` off this outcome, exactly as it
    /// does for [`Self::on_game_action_list_received`].
    ///
    /// # Panics
    ///
    /// Panics if the dealt deck index or total deck size does not fit in a `u8` (never the case
    /// for real Hanabi games).
    pub async fn on_card_identities_received(
        &mut self,
        payload: CardIdentitiesPayload,
        websocket_client: &mut dyn OutboundSink,
    ) -> TurnOutcome {
        if !matches!(self, BotState::WaitingForGameActionList { .. }) {
            tracing::warn!("Received cardIdentities in unexpected state, ignoring");
            return TurnOutcome::Waiting;
        }
        // Move the owned `tables`/`init_data`/`static_data` out of the current state so they
        // can be threaded into `Playing` without cloning the (potentially large) game data.
        let BotState::WaitingForGameActionList {
            tables,
            table_id,
            init_data,
            static_data,
        } = std::mem::replace(self, BotState::Idle { tables: Vec::new() })
        else {
            unreachable!("matched WaitingForCardIdentities above")
        };

        let stacks_size = static_data.variant.stacks_size as usize;
        let actual_deck: Vec<VariantCardId> = payload
            .card_identities
            .iter()
            .map(|c| c.suit_index * stacks_size + (c.rank as usize - 1))
            .collect();

        let num_players = static_data.number_of_players as usize;
        let hand_size = hand_size_for(static_data.number_of_players);

        let mut game = KnowledgeAwareGameState::new(static_data.clone());
        for player in 0..num_players {
            game.table_state.active_player_index = player;
            for slot in 0..hand_size {
                let deck_idx = CardDeckIndex::try_from(player * hand_size + slot)
                    .expect("deck index fits in u8");
                let card_id = actual_deck[deck_idx as usize];
                game.update_with_draw_action_of_specific_card(player, deck_idx, card_id);
            }
        }
        game.table_state.active_player_index = init_data.options.starting_player;
        game.next_deck_index = u8::try_from(num_players * hand_size).expect("deck size fits in u8");

        let our_player_index = init_data.our_player_index;

        if let Err(e) = websocket_client
            .send(&OutboundWebsocketMessage::Loaded(table_id))
            .await
        {
            tracing::error!("Failed to send Loaded: {e}");
        }

        tracing::info!("Game ready: we are player {our_player_index}");

        *self = BotState::Playing {
            tables,
            table_id,
            our_player_index,
            game,
            actual_deck,
            static_data,
        };

        // No actions follow the deal here, so the game cannot be over; the only question is
        // whether the starting player is us.
        self.turn_outcome(false)
    }

    /// Apply all network actions from a `gameActionList`.
    pub fn apply_action_list(
        &mut self,
        payload: &GameActionListPayload,
        convention_set: &HGroupConventionSet,
    ) -> TurnOutcome {
        // Pre-populate actual_deck from every Draw action in the list so that
        // GroundTruthResolver can look up post-deal draw identities when processing
        // the Play/Discard that precedes each Draw in the same list.
        if let BotState::Playing {
            actual_deck,
            static_data,
            ..
        } = self
        {
            for action in &payload.list {
                if let GameActionDto::Draw {
                    order,
                    suit_index,
                    rank,
                    ..
                } = action
                {
                    record_revealed_identity(actual_deck, static_data, *order, *suit_index, *rank);
                }
            }
        }
        let mut game_over = false;
        for action in &payload.list {
            game_over |= self.apply_network_action(action, convention_set);
        }
        self.turn_outcome(game_over)
    }

    /// Apply a single `gameAction`.
    pub fn apply_game_action(
        &mut self,
        payload: &GameActionPayload,
        convention_set: &HGroupConventionSet,
    ) -> TurnOutcome {
        let game_over = self.apply_network_action(&payload.action, convention_set);
        self.turn_outcome(game_over)
    }

    fn turn_outcome(&self, game_over: bool) -> TurnOutcome {
        if game_over {
            TurnOutcome::GameOver
        } else if self.is_our_turn() {
            TurnOutcome::OurTurn
        } else {
            TurnOutcome::Waiting
        }
    }

    /// Apply a single network action. Returns `true` if the action ended the game.
    fn apply_network_action(
        &mut self,
        action: &GameActionDto,
        convention_set: &HGroupConventionSet,
    ) -> bool {
        let BotState::Playing {
            game,
            actual_deck,
            static_data,
            ..
        } = self
        else {
            return false;
        };

        match action {
            GameActionDto::Draw {
                player_index,
                order,
                suit_index,
                rank,
            } => {
                let stacks_size = static_data.variant.stacks_size as usize;
                let deck_idx = CardDeckIndex::try_from(*order).expect("deck index fits in u8");
                if *suit_index >= 0 && *rank >= 0 {
                    // Known card (opponent's draw): reveal identity to all other players and
                    // record in actual_deck for future resolver use.
                    let card_id = *suit_index as usize * stacks_size + (*rank as usize - 1);
                    if *order >= actual_deck.len() {
                        actual_deck.resize(*order + 1, 0);
                    }
                    actual_deck[*order] = card_id;
                    game.team_knowledge
                        .update_with_card_drawn(*player_index, deck_idx, card_id);
                }
                // Unknown card (our draw): nothing to reveal; the hand slot was already
                // added via update_with_unkown_card_draw inside apply_play/apply_discard.
            }
            GameActionDto::Play {
                player_index,
                order,
                suit_index,
                rank,
            } => {
                // The server reveals the played card's identity in the action itself. Record it
                // so GroundTruthResolver can resolve it — for our own cards the initial-deal Draw
                // was hidden (-1/-1), so this is the first time we learn what the card was.
                record_revealed_identity(actual_deck, static_data, *order, *suit_index, *rank);
                let turn = game.table_state.current_turn;
                let ga = GameAction::Play {
                    player_index: *player_index,
                    card_deck_index: *order as CardDeckIndex,
                    turn,
                };
                let mut resolver = GroundTruthResolver::new(actual_deck);
                game.apply(&ga, convention_set, &mut resolver);
                game.advance_turn();
            }
            GameActionDto::Discard {
                player_index,
                order,
                failed,
                suit_index,
                rank,
            } => {
                // Likewise, the server reveals the discarded (or misplayed) card's identity here.
                record_revealed_identity(actual_deck, static_data, *order, *suit_index, *rank);
                let turn = game.table_state.current_turn;
                // A `failed` discard on hanab.live is a misplay (bomb): the player *attempted a
                // play* that turned out unplayable. Model it as a `Play` so the engine runs it
                // through the play path — the ground-truth resolver finds the card unplayable,
                // records a strike, and moves it to the discard pile (and, correctly, does not
                // refund a clue token the way a real discard would). Routing it as a plain
                // `Discard` would silently drop the strike and over-credit a clue, diverging from
                // the server (this is the divergence the #10 Strike canary catches).
                let ga = if *failed {
                    GameAction::Play {
                        player_index: *player_index,
                        card_deck_index: *order as CardDeckIndex,
                        turn,
                    }
                } else {
                    GameAction::Discard {
                        player_index: *player_index,
                        card_deck_index: *order as CardDeckIndex,
                        turn,
                    }
                };
                let mut resolver = GroundTruthResolver::new(actual_deck);
                game.apply(&ga, convention_set, &mut resolver);
                game.advance_turn();
            }
            GameActionDto::Clue {
                clue, target, list, ..
            } => {
                let clue_type = if clue.clue_type == 0 {
                    ClueType::Color
                } else {
                    ClueType::Rank
                };
                let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> =
                    list.iter().map(|&o| o as CardDeckIndex).collect();
                let turn = game.table_state.current_turn;
                let ga = GameAction::Clue {
                    player_index: *target,
                    touched_card_deck_indexes: touched,
                    clue: Clue {
                        clue_type,
                        clue_value: clue.value,
                    },
                    turn,
                };
                let mut resolver = GroundTruthResolver::new(actual_deck);
                game.apply(&ga, convention_set, &mut resolver);
                game.advance_turn();
            }
            GameActionDto::Turn {
                current_player_index,
                ..
            } => {
                // The server is authoritative for turn order; adopting its `current_player_index`
                // *is* the resync. In normal play our local `advance_turn` already matches, so a
                // mismatch here means our view diverged (e.g. an end-of-deck final-round skip or a
                // missed/duplicated action). Surface it at `warn` rather than silently correcting,
                // so a persistent desync is visible instead of being papered over every turn (#9).
                if game.table_state.active_player_index != *current_player_index {
                    tracing::warn!(
                        "Turn desync: server says active player {}, local had {} — resyncing to server",
                        current_player_index,
                        game.table_state.active_player_index
                    );
                    game.table_state.active_player_index = *current_player_index;
                }
            }
            GameActionDto::Status { score, .. } => {
                // Desync canary (#10). The server's `Status` follows the Play/Discard it
                // describes in the action stream, so by the time it arrives our running score
                // should already match it. A mismatch is the earliest, clearest signal that the
                // engine's view has diverged from the server — surface it rather than acting on a
                // stale view. The engine remains the source of truth for play; this only watches.
                let local_score = usize::from(game.table_state.score(&static_data.variant));
                if local_score != *score {
                    tracing::warn!(
                        "Score desync: server reports {score}, engine has {local_score}"
                    );
                }
            }
            GameActionDto::Strike { num, .. } => {
                // Desync canary (#10). `num` is the server's running strike count (1-indexed);
                // it should equal the engine's strike tokens once the misplay is applied.
                let local_strikes = usize::from(game.table_state.strike_tokens);
                if local_strikes != *num {
                    tracing::warn!(
                        "Strike desync: server reports strike #{num}, engine has {local_strikes}"
                    );
                }
            }
            GameActionDto::GameOver {
                end_condition,
                player_index,
            } => {
                tracing::info!(
                    "Game over (end_condition={end_condition}, player_index={player_index})"
                );
                return true;
            }
        }
        false
    }

    fn is_our_turn(&self) -> bool {
        if let BotState::Playing {
            game,
            our_player_index,
            ..
        } = self
        {
            game.table_state.active_player_index == *our_player_index
        } else {
            false
        }
    }

    /// Compute the engine's recommended action and send it to the server.
    /// Returns `true` if an action was sent.
    ///
    /// This does **not** mutate `game`. hanab.live echoes every action back to all
    /// players (including the actor) as a `gameAction`; that echo is what advances our
    /// local state via [`Self::apply_network_action`]. Applying locally here as well
    /// would double-apply our own move and permanently desync from the server.
    pub async fn take_turn(
        &mut self,
        convention_set: &HGroupConventionSet,
        strategy: &TreeActionSelectionStrategy,
        websocket_client: &mut dyn OutboundSink,
    ) -> bool {
        let BotState::Playing {
            table_id,
            our_player_index,
            game,
            ..
        } = self
        else {
            return false;
        };
        if game.table_state.active_player_index != *our_player_index {
            return false;
        }

        let engine_action = {
            let pov = game.player_pov(*our_player_index);
            strategy.select_active_player_action(&pov, convention_set)
        };

        tracing::info!("Taking action: {engine_action:?}");

        let Some(msg) = game_action_to_outbound(*table_id, &engine_action) else {
            tracing::error!("Engine produced an unexpected Draw action");
            return false;
        };

        if let Err(e) = websocket_client.send(&msg).await {
            tracing::error!("Failed to send game action: {e}");
            return false;
        }

        true
    }

    /// Handle the end of a game: leave the table (`tableUnattend`) and return to
    /// `Idle` so the bot can play another game during this process's lifetime.
    pub async fn on_game_over(&mut self, websocket_client: &mut dyn OutboundSink) {
        let BotState::Playing { table_id, .. } = self else {
            return;
        };
        let table_id = *table_id;

        let msg = OutboundWebsocketMessage::TableUnattend(table_id);
        if let Err(e) = websocket_client.send(&msg).await {
            tracing::error!("Failed to send tableUnattend after game over: {e}");
        }
        tracing::info!("Game over, leaving table {table_id} and returning to idle");
        let tables = std::mem::take(self.tables_mut());
        *self = BotState::Idle { tables };
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use crate::engine::convention::hgroup::h_group_convention_set::{
        HGroupConventionSet, HGroupLevel,
    };
    use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
    use crate::game::action::game_action::GameAction;
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;
    use crate::game::static_game_data::StaticGameData;
    use crate::game::variant::test_variants::NO_VARIANT;
    use crate::hanablive::action::ActionType;
    use crate::hanablive::client::websocket_client::WebSocketClient;
    use crate::hanablive::dto::payload::card_identity::{CardIdentitiesPayload, CardIdentity};
    use crate::hanablive::dto::payload::game_action::{
        GameActionDto, GameActionListPayload, GameActionPayload,
    };
    use crate::hanablive::dto::payload::game_init_data::GameInitData;
    use crate::hanablive::dto::payload::game_options::GameOptions;
    use crate::hanablive::inbound_websocket_message::InboundWebsocketMessage;
    use crate::hanablive::outbound_websocket_message::OutboundWebsocketMessage;

    use super::{BotState, TurnOutcome, game_action_to_outbound};

    const TABLE_ID: u32 = 42;

    fn two_player_playing_state(our_player_index: usize) -> BotState {
        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let game = KnowledgeAwareGameState::new(static_data.clone());
        BotState::Playing {
            tables: vec![],
            table_id: TABLE_ID,
            our_player_index,
            game,
            actual_deck: vec![],
            static_data,
        }
    }

    /// A canonical 50-card No Variant deck, in `(suit * 5 + rank - 1)` id order, with
    /// copy counts 3,2,2,2,1 per suit. `deck[0]` is therefore R1 (playable on turn 1).
    fn canonical_deck() -> Vec<crate::game::card::VariantCardId> {
        const COPIES_BY_RANK: [usize; 5] = [3, 2, 2, 2, 1];
        let mut deck = Vec::new();
        for suit in 0..5 {
            for rank in 0..5 {
                let card_id = suit * 5 + rank;
                for _ in 0..COPIES_BY_RANK[rank] {
                    deck.push(card_id);
                }
            }
        }
        deck
    }

    /// A 2-player `Playing` state with both hands dealt from [`canonical_deck`],
    /// `next_deck_index` past the dealt cards, and `player 0` to act.
    fn dealt_two_player_playing_state(our_player_index: usize) -> BotState {
        use crate::game::card::CardDeckIndex;

        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let actual_deck = canonical_deck();
        let num_players = 2;
        let hand_size = 5;

        let mut game = KnowledgeAwareGameState::new(static_data.clone());
        for player in 0..num_players {
            game.table_state.active_player_index = player;
            for slot in 0..hand_size {
                let deck_idx = CardDeckIndex::try_from(player * hand_size + slot)
                    .expect("deck index fits in u8");
                game.update_with_draw_action_of_specific_card(
                    player,
                    deck_idx,
                    actual_deck[deck_idx as usize],
                );
            }
        }
        game.table_state.active_player_index = 0;
        game.next_deck_index = u8::try_from(num_players * hand_size).expect("deck size fits in u8");

        BotState::Playing {
            tables: vec![],
            table_id: TABLE_ID,
            our_player_index,
            game,
            actual_deck,
            static_data,
        }
    }

    /// Point #1: hanab.live echoes our own action back as a `gameAction`. Because
    /// `take_turn` no longer mutates `game`, applying that echo must advance state
    /// exactly once — the played card scores once and a single replacement is drawn.
    #[test]
    fn own_action_echo_is_applied_exactly_once() {
        let mut state = dealt_two_player_playing_state(0);

        let (score_before, deck_index_before, turn_before) = {
            let BotState::Playing {
                game, static_data, ..
            } = &state
            else {
                unreachable!()
            };
            (
                game.table_state.score(&static_data.variant),
                game.next_deck_index,
                game.table_state.current_turn,
            )
        };
        assert_eq!(score_before, 0);

        // The server echo of our own "play deck index 0" (R1, playable), followed by the
        // Turn message handing play to the opponent.
        let echo = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Play {
                player_index: 0,
                order: 0,
                suit_index: 0,
                rank: 1,
            },
        };
        let outcome = state.apply_game_action(&echo, &convention_set());
        assert_eq!(
            outcome,
            TurnOutcome::Waiting,
            "play does not change active player by itself"
        );

        let BotState::Playing {
            game, static_data, ..
        } = &state
        else {
            unreachable!()
        };
        assert_eq!(
            game.table_state.score(&static_data.variant),
            score_before + 1,
            "R1 should be on the stack exactly once"
        );
        assert_eq!(
            game.next_deck_index,
            deck_index_before + 1,
            "exactly one replacement card should be drawn"
        );
        assert_eq!(
            game.table_state.current_turn,
            turn_before + 1,
            "the turn counter should advance exactly once"
        );
    }

    /// A `failed` discard is a misplay (bomb): the engine must record a strike and leave the
    /// score untouched, matching the server. (Player 0 is dealt `[R1,R1,R1,R2,R2]` at indices
    /// 0–4; with empty stacks, failing the R2 at index 3 is a guaranteed misplay.) A subsequent
    /// `Strike { num: 1 }` then reconciles cleanly against the engine (#10).
    #[test]
    fn failed_discard_records_a_strike_without_scoring() {
        let mut state = dealt_two_player_playing_state(1);

        let bomb = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Discard {
                player_index: 0,
                order: 3,
                suit_index: 0,
                rank: 2,
                failed: true,
            },
        };
        state.apply_game_action(&bomb, &convention_set());

        let BotState::Playing {
            game, static_data, ..
        } = &state
        else {
            unreachable!()
        };
        assert_eq!(
            game.table_state.strike_tokens, 1,
            "a bomb must record exactly one strike"
        );
        assert_eq!(
            game.table_state.score(&static_data.variant),
            0,
            "a misplay must not advance the score"
        );
    }

    fn convention_set() -> HGroupConventionSet {
        HGroupConventionSet::for_level(HGroupLevel::One, &NO_VARIANT)
    }

    // --- game_action_to_outbound ---

    #[test]
    fn outbound_play_uses_deck_index_as_target() {
        let action = GameAction::Play {
            player_index: 0,
            card_deck_index: 5,
            turn: 0,
        };
        let msg = game_action_to_outbound(TABLE_ID, &action).unwrap();
        assert!(matches!(
            msg,
            OutboundWebsocketMessage::GameAction {
                action_type: ActionType::Play,
                target: 5,
                value: None,
                ..
            }
        ));
    }

    #[test]
    fn outbound_discard_uses_deck_index_as_target() {
        let action = GameAction::Discard {
            player_index: 0,
            card_deck_index: 3,
            turn: 0,
        };
        let msg = game_action_to_outbound(TABLE_ID, &action).unwrap();
        assert!(matches!(
            msg,
            OutboundWebsocketMessage::GameAction {
                action_type: ActionType::Discard,
                target: 3,
                value: None,
                ..
            }
        ));
    }

    #[test]
    fn outbound_color_clue_carries_suit_as_value() {
        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec![],
            clue: Clue {
                clue_type: ClueType::Color,
                clue_value: 2,
            },
            turn: 0,
        };
        let msg = game_action_to_outbound(TABLE_ID, &action).unwrap();
        assert!(matches!(
            msg,
            OutboundWebsocketMessage::GameAction {
                action_type: ActionType::ColorClue,
                target: 1,
                value: Some(2),
                ..
            }
        ));
    }

    #[test]
    fn outbound_rank_clue_carries_rank_as_value() {
        let action = GameAction::Clue {
            player_index: 1,
            touched_card_deck_indexes: smallvec![],
            clue: Clue {
                clue_type: ClueType::Rank,
                clue_value: 4,
            },
            turn: 0,
        };
        let msg = game_action_to_outbound(TABLE_ID, &action).unwrap();
        assert!(matches!(
            msg,
            OutboundWebsocketMessage::GameAction {
                action_type: ActionType::RankClue,
                target: 1,
                value: Some(4),
                ..
            }
        ));
    }

    #[test]
    fn outbound_draw_returns_none() {
        let action = GameAction::Draw {
            player_index: 0,
            card_deck_index: 0,
        };
        assert!(game_action_to_outbound(TABLE_ID, &action).is_none());
    }

    // --- variant guard (#4) ---

    #[test]
    fn no_variant_is_supported() {
        assert!(super::is_supported_variant("No Variant"));
    }

    #[test]
    fn other_variants_are_unsupported() {
        assert!(!super::is_supported_variant("6 Suits"));
        assert!(!super::is_supported_variant("Black (5 Suits)"));
        assert!(!super::is_supported_variant(""));
    }

    // --- apply_game_action / is_our_turn ---

    #[test]
    fn turn_message_syncs_active_player_and_returns_true_when_our_turn() {
        let mut state = two_player_playing_state(1);
        let payload = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Turn {
                num: 1,
                current_player_index: 1,
            },
        };
        assert_eq!(
            state.apply_game_action(&payload, &convention_set()),
            TurnOutcome::OurTurn
        );
    }

    #[test]
    fn turn_message_returns_false_when_not_our_turn() {
        let mut state = two_player_playing_state(0);
        let payload = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Turn {
                num: 1,
                current_player_index: 1,
            },
        };
        assert_eq!(
            state.apply_game_action(&payload, &convention_set()),
            TurnOutcome::Waiting
        );
    }

    /// Point #9: the server is authoritative for turn order. If our local view of the active
    /// player diverges (e.g. an end-of-deck final-round skip we modeled differently), the
    /// `Turn` message must resync us to the server's `current_player_index`.
    #[test]
    fn turn_message_resyncs_diverged_active_player_to_server() {
        let mut state = two_player_playing_state(0);
        // Simulate a local divergence: our view thinks it is player 1's turn...
        if let BotState::Playing { game, .. } = &mut state {
            game.table_state.active_player_index = 1;
        }
        // ...but the server says it is player 0's turn.
        let payload = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Turn {
                num: 7,
                current_player_index: 0,
            },
        };
        assert_eq!(
            state.apply_game_action(&payload, &convention_set()),
            TurnOutcome::OurTurn
        );
        let BotState::Playing { game, .. } = &state else {
            unreachable!()
        };
        assert_eq!(
            game.table_state.active_player_index, 0,
            "the server's current_player_index wins over the diverged local value"
        );
    }

    /// Point #9: deck-empty final-round flow. Once the deck is empty no draws occur, and the
    /// server drives the remaining players' final turns purely via `Turn` messages. Feeding
    /// that sequence must walk the active player exactly as the server dictates and only yield
    /// `OurTurn` when it is genuinely our seat's turn.
    #[test]
    fn final_round_turns_follow_server_without_drawing() {
        // 2-player game, we are player 1. Drive the final-round turn handoffs.
        let mut state = two_player_playing_state(1);
        let deck_index_before = match &state {
            BotState::Playing { game, .. } => game.next_deck_index,
            _ => unreachable!(),
        };

        // Server: player 0 acts, then hands the (final) turn to us (player 1).
        let to_p0 = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Turn {
                num: 18,
                current_player_index: 0,
            },
        };
        assert_eq!(
            state.apply_game_action(&to_p0, &convention_set()),
            TurnOutcome::Waiting
        );

        let to_us = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Turn {
                num: 19,
                current_player_index: 1,
            },
        };
        assert_eq!(
            state.apply_game_action(&to_us, &convention_set()),
            TurnOutcome::OurTurn
        );

        // No card was drawn while resyncing turns — the deck index is untouched.
        let BotState::Playing { game, .. } = &state else {
            unreachable!()
        };
        assert_eq!(
            game.next_deck_index, deck_index_before,
            "Turn messages must not draw cards (the deck is empty in the final round)"
        );
    }

    #[test]
    fn apply_action_list_returns_true_on_our_turn() {
        let mut state = two_player_playing_state(1);
        let payload = GameActionListPayload {
            table_id: TABLE_ID,
            list: vec![GameActionDto::Turn {
                num: 0,
                current_player_index: 1,
            }],
        };
        assert_eq!(
            state.apply_action_list(&payload, &convention_set()),
            TurnOutcome::OurTurn
        );
    }

    #[test]
    fn apply_action_list_with_status_and_strike_does_not_crash() {
        let mut state = two_player_playing_state(0);
        let payload = GameActionListPayload {
            table_id: TABLE_ID,
            list: vec![
                GameActionDto::Status {
                    clues: 8,
                    score: 0,
                    max_score: 25,
                },
                GameActionDto::Strike {
                    num: 1,
                    turn: 3,
                    order: 5,
                },
                GameActionDto::Turn {
                    num: 4,
                    current_player_index: 0,
                },
            ],
        };
        assert_eq!(
            state.apply_action_list(&payload, &convention_set()),
            TurnOutcome::OurTurn
        );
    }

    /// Point #10: the `Status` desync canary must accept a server score that matches the
    /// engine's. After our R1 play echo the engine's score is 1; a `Status { score: 1 }`
    /// reconciles cleanly and leaves it our opponent's turn (no spurious state change).
    #[test]
    fn status_matching_engine_score_reconciles_cleanly() {
        let mut state = dealt_two_player_playing_state(0);
        let echo = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Play {
                player_index: 0,
                order: 0,
                suit_index: 0,
                rank: 1,
            },
        };
        assert_eq!(
            state.apply_game_action(&echo, &convention_set()),
            TurnOutcome::Waiting
        );

        let status = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Status {
                clues: 8,
                score: 1,
                max_score: 25,
            },
        };
        assert_eq!(
            state.apply_game_action(&status, &convention_set()),
            TurnOutcome::Waiting
        );

        let BotState::Playing {
            game, static_data, ..
        } = &state
        else {
            unreachable!()
        };
        assert_eq!(
            game.table_state.score(&static_data.variant),
            1,
            "Status must not mutate score"
        );
    }

    #[test]
    fn apply_game_action_on_non_playing_state_returns_false() {
        let mut state = BotState::Idle { tables: vec![] };
        let payload = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Turn {
                num: 0,
                current_player_index: 0,
            },
        };
        assert_eq!(
            state.apply_game_action(&payload, &convention_set()),
            TurnOutcome::Waiting
        );
    }

    #[test]
    fn game_over_action_yields_game_over_outcome() {
        let mut state = dealt_two_player_playing_state(0);
        let payload = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::GameOver {
                end_condition: 1,
                player_index: -1,
            },
        };
        assert_eq!(
            state.apply_game_action(&payload, &convention_set()),
            TurnOutcome::GameOver,
            "a gameOver action must surface as a terminal outcome"
        );
    }

    #[test]
    fn game_over_in_action_list_overrides_our_turn() {
        // Even if it would otherwise be our turn after the list, a gameOver wins.
        let mut state = two_player_playing_state(1);
        let payload = GameActionListPayload {
            table_id: TABLE_ID,
            list: vec![
                GameActionDto::Turn {
                    num: 0,
                    current_player_index: 1,
                },
                GameActionDto::GameOver {
                    end_condition: 2,
                    player_index: 0,
                },
            ],
        };
        assert_eq!(
            state.apply_action_list(&payload, &convention_set()),
            TurnOutcome::GameOver
        );
    }

    // --- async transition tests (require a real in-memory WebSocket) ---

    fn minimal_init_data(
        table_id: u32,
        variant_name: &str,
        num_players: usize,
        our_player_index: usize,
    ) -> GameInitData {
        GameInitData {
            table_id,
            player_names: (0..num_players).map(|i| format!("Player{i}")).collect(),
            our_player_index,
            spectating: false,
            shadowing: false,
            replay: false,
            database_id: -1,
            has_custom_seed: false,
            seed: String::from("test-seed"),
            datetime_started: chrono::Utc::now(),
            datetime_finished: chrono::Utc::now(),
            options: GameOptions {
                max_players: None,
                num_players,
                starting_player: 0,
                variant_id: 0,
                variant_name: variant_name.to_owned(),
                table_name: None,
                timed: false,
                time_base: 0,
                time_per_turn: 0,
                speedrun: false,
                card_cycle: false,
                deck_plays: false,
                empty_clues: false,
                one_extra_card: false,
                one_less_card: false,
                all_or_nothing: false,
                detrimental_characters: false,
            },
            character_assignments: vec![],
            character_metadata: vec![],
            shared_replay: false,
            shared_replay_leader: String::new(),
            shared_replay_segment: 0,
            shared_replay_eff_mod: 0,
            paused: false,
            pause_player_index: -1,
            pause_queued: false,
        }
    }

    /// Convert a canonical deck (Vec<VariantCardId>) back to wire card identities for No Variant
    /// (stacks_size = 5): suit_index = id / 5, rank = id % 5 + 1.
    fn card_identities_from_deck(deck: &[crate::game::card::VariantCardId]) -> Vec<CardIdentity> {
        deck.iter()
            .map(|&id| CardIdentity {
                suit_index: id / 5,
                rank: (id % 5 + 1) as u8,
            })
            .collect()
    }

    /// Point #4 (safety net): `on_init_received` must refuse an unsupported variant even if the
    /// bot somehow reached `WaitingForInit` without the join-time guard catching it. The state
    /// must revert to `Idle` and send `tableUnattend`.
    #[tokio::test]
    async fn on_init_received_unsupported_variant_reverts_to_idle() {
        let mut ws = WebSocketClient::for_test().await;
        let mut state = BotState::WaitingForInit {
            tables: vec![],
            table_id: TABLE_ID,
        };
        let init_data = minimal_init_data(TABLE_ID, "Duck (5 Suits)", 2, 0);

        state.on_init_received(init_data, &mut ws).await;

        assert!(
            matches!(state, BotState::Idle { .. }),
            "unsupported variant must revert to Idle so the bot can join another game"
        );
    }

    /// Point #3 (hand-size formula): a 6-player game must deal exactly 3 cards per player
    /// (hand_size_for(6) == 3), leaving next_deck_index at 18 (6 × 3) after the initial deal.
    #[tokio::test]
    async fn on_card_identities_received_six_player_deals_three_cards_each() {
        let mut ws = WebSocketClient::for_test().await;
        let init_data = minimal_init_data(TABLE_ID, "No Variant", 6, 0);
        let static_data = StaticGameData {
            number_of_players: 6,
            variant: NO_VARIANT,
        };
        let mut state = BotState::WaitingForGameActionList {
            tables: vec![],
            table_id: TABLE_ID,
            init_data,
            static_data,
        };

        let deck = canonical_deck(); // 50 No Variant cards; only the first 18 are dealt
        let payload = CardIdentitiesPayload {
            table_id: TABLE_ID,
            card_identities: card_identities_from_deck(&deck),
        };
        let _ = state.on_card_identities_received(payload, &mut ws).await;

        let BotState::Playing { game, .. } = &state else {
            panic!("expected Playing state for 6-player game");
        };
        assert_eq!(
            game.next_deck_index, 18,
            "6 players × 3 cards per hand = 18 dealt"
        );
    }

    /// Regression: when the deal arrives via `cardIdentities` and we are the starting player,
    /// nothing else will prompt us (it is our turn, so no opponent action is coming). Dealing
    /// must therefore surface `OurTurn` so the caller drives `take_turn`. Previously this path
    /// returned no outcome and the bot sat idle whenever it was first to act.
    #[tokio::test]
    async fn on_card_identities_received_returns_our_turn_when_we_start() {
        let mut ws = WebSocketClient::for_test().await;
        // starting_player defaults to 0; make us player 0 so we are first to act.
        let init_data = minimal_init_data(TABLE_ID, "No Variant", 2, 0);
        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let mut state = BotState::WaitingForGameActionList {
            tables: vec![],
            table_id: TABLE_ID,
            init_data,
            static_data,
        };

        let payload = CardIdentitiesPayload {
            table_id: TABLE_ID,
            card_identities: card_identities_from_deck(&canonical_deck()),
        };
        let outcome = state.on_card_identities_received(payload, &mut ws).await;

        assert_eq!(
            outcome,
            TurnOutcome::OurTurn,
            "the starting player must be prompted to act after the cardIdentities deal"
        );
    }

    /// Counterpart to the above: when we are *not* the starting player, the `cardIdentities`
    /// deal must leave us `Waiting` (the opponent acts first, and their action will arrive as a
    /// later `gameAction`).
    #[tokio::test]
    async fn on_card_identities_received_returns_waiting_when_opponent_starts() {
        let mut ws = WebSocketClient::for_test().await;
        // starting_player defaults to 0, but we are player 1, so it is not our turn yet.
        let init_data = minimal_init_data(TABLE_ID, "No Variant", 2, 1);
        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let mut state = BotState::WaitingForGameActionList {
            tables: vec![],
            table_id: TABLE_ID,
            init_data,
            static_data,
        };

        let payload = CardIdentitiesPayload {
            table_id: TABLE_ID,
            card_identities: card_identities_from_deck(&canonical_deck()),
        };
        let outcome = state.on_card_identities_received(payload, &mut ws).await;

        assert_eq!(
            outcome,
            TurnOutcome::Waiting,
            "when the opponent starts, dealing must not prompt us to act"
        );
    }

    /// End-to-end fixture test: parse the real `card_identities.json` fixture to deal a 2-player
    /// game, then verify the deck pointer is correct and that a subsequent Play action scores
    /// exactly once and draws one replacement. This catches deck-index desyncs between the initial
    /// deal and the first network action.
    #[tokio::test]
    async fn fixture_card_identities_then_play_scores_and_draws() {
        // Load and parse the real fixture.
        let raw = include_str!("../test_fixtures/card_identities.json");
        let InboundWebsocketMessage::CardIdentities(payload) = InboundWebsocketMessage::parse(raw)
        else {
            panic!("failed to parse card_identities.json fixture as CardIdentities");
        };

        let mut ws = WebSocketClient::for_test().await;

        // Treat the 50-card fixture as a 2-player No Variant game; ourPlayerIndex = 0.
        let init_data = minimal_init_data(TABLE_ID, "No Variant", 2, 0);
        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let mut state = BotState::WaitingForGameActionList {
            tables: vec![],
            table_id: TABLE_ID,
            init_data,
            static_data,
        };

        let outcome = state.on_card_identities_received(payload, &mut ws).await;
        assert_eq!(
            outcome,
            TurnOutcome::OurTurn,
            "starting_player == our_player_index == 0, so dealing must hand us the turn"
        );

        // Verify: 2 players × 5 cards = 10 dealt, next pointer sits at 10.
        let BotState::Playing { game, .. } = &state else {
            panic!("expected Playing state after cardIdentities");
        };
        assert_eq!(
            game.next_deck_index, 10,
            "2p × 5 cards = 10 dealt from fixture"
        );

        // The fixture's card at deck order 1 is {suitIndex: 2, rank: 1} → G1 in No Variant,
        // which is immediately playable on empty stacks.
        let play = GameActionPayload {
            table_id: TABLE_ID,
            action: GameActionDto::Play {
                player_index: 0,
                order: 1,
                suit_index: 2,
                rank: 1,
            },
        };
        state.apply_game_action(&play, &convention_set());

        let BotState::Playing {
            game, static_data, ..
        } = &state
        else {
            unreachable!()
        };
        assert_eq!(
            game.table_state.score(&static_data.variant),
            1,
            "G1 from fixture must score exactly once"
        );
        assert_eq!(
            game.next_deck_index, 11,
            "one replacement must be drawn (deck pointer 10 → 11)"
        );
    }

    // --- on_game_action_list_received ---

    /// Deals a 2-player game via a gameActionList with 10 Draw actions (5 each).
    /// Player 0 is us (suitIndex/rank = -1); player 1's cards are visible.
    #[tokio::test]
    async fn on_game_action_list_received_deals_initial_hands() {
        let mut ws = WebSocketClient::for_test().await;
        let init_data = minimal_init_data(TABLE_ID, "No Variant", 2, 0);
        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let mut state = BotState::WaitingForGameActionList {
            tables: vec![],
            table_id: TABLE_ID,
            init_data,
            static_data,
        };

        // 5 draws for player 0 (unknown), then 5 draws for player 1 (known).
        let mut list: Vec<GameActionDto> = (0..5usize)
            .map(|order| GameActionDto::Draw {
                player_index: 0,
                order,
                suit_index: -1,
                rank: -1,
            })
            .collect();
        list.extend((5..10usize).map(|order| GameActionDto::Draw {
            player_index: 1,
            order,
            suit_index: 0,
            rank: (order - 4) as i32, // R1..R5
        }));
        list.push(GameActionDto::Turn {
            num: 0,
            current_player_index: 0,
        });
        let payload = GameActionListPayload {
            table_id: TABLE_ID,
            list,
        };

        let outcome = state
            .on_game_action_list_received(&payload, &convention_set(), &mut ws)
            .await;

        assert_eq!(outcome, TurnOutcome::OurTurn, "Turn 0 is player 0's turn");
        let BotState::Playing { game, .. } = &state else {
            panic!("expected Playing state");
        };
        assert_eq!(game.next_deck_index, 10, "10 cards dealt");
        assert_eq!(
            game.table_state.active_player_index, 0,
            "active player synced from Turn action"
        );
    }

    /// When the bot is the starting player and the initial GameActionList contains only Draw
    /// actions (no Turn message), the outcome must be OurTurn, not Waiting.
    #[tokio::test]
    async fn on_game_action_list_draws_only_returns_our_turn_when_we_start() {
        let mut ws = WebSocketClient::for_test().await;
        // starting_player = 0 = our_player_index
        let init_data = minimal_init_data(TABLE_ID, "No Variant", 2, 0);
        let static_data = StaticGameData {
            number_of_players: 2,
            variant: NO_VARIANT,
        };
        let mut state = BotState::WaitingForGameActionList {
            tables: vec![],
            table_id: TABLE_ID,
            init_data,
            static_data,
        };

        // Draw-only list: 5 unknown for us, 5 known for opponent.
        let mut list: Vec<GameActionDto> = (0..5usize)
            .map(|order| GameActionDto::Draw {
                player_index: 0,
                order,
                suit_index: -1,
                rank: -1,
            })
            .collect();
        list.extend((5..10usize).map(|order| GameActionDto::Draw {
            player_index: 1,
            order,
            suit_index: 0,
            rank: (order - 4) as i32,
        }));
        let payload = GameActionListPayload {
            table_id: TABLE_ID,
            list,
        };

        let outcome = state
            .on_game_action_list_received(&payload, &convention_set(), &mut ws)
            .await;

        assert_eq!(
            outcome,
            TurnOutcome::OurTurn,
            "starting player must get OurTurn even when no Turn action is in the list"
        );
    }

    /// When already Playing, on_game_action_list_received delegates to apply_action_list.
    #[tokio::test]
    async fn on_game_action_list_received_delegates_when_playing() {
        let mut ws = WebSocketClient::for_test().await;
        let mut state = two_player_playing_state(1);
        let payload = GameActionListPayload {
            table_id: TABLE_ID,
            list: vec![GameActionDto::Turn {
                num: 0,
                current_player_index: 1,
            }],
        };
        let outcome = state
            .on_game_action_list_received(&payload, &convention_set(), &mut ws)
            .await;
        assert_eq!(outcome, TurnOutcome::OurTurn);
    }
}

fn game_action_to_outbound(
    table_id: TableId,
    action: &GameAction,
) -> Option<OutboundWebsocketMessage> {
    match action {
        GameAction::Play {
            card_deck_index, ..
        } => Some(OutboundWebsocketMessage::GameAction {
            table_id,
            action_type: ActionType::Play,
            target: *card_deck_index as usize,
            value: None,
        }),
        GameAction::Discard {
            card_deck_index, ..
        } => Some(OutboundWebsocketMessage::GameAction {
            table_id,
            action_type: ActionType::Discard,
            target: *card_deck_index as usize,
            value: None,
        }),
        GameAction::Clue {
            player_index, clue, ..
        } => {
            let action_type = match clue.clue_type {
                ClueType::Color => ActionType::ColorClue,
                ClueType::Rank => ActionType::RankClue,
            };
            // TODO(#12, variants): `clue.clue_value` is sent verbatim as the wire `value`. For a
            // color clue that value is a suit/color index, which is only identical to the engine's
            // index under "No Variant". Once non-"No Variant" variants are supported (#4), this
            // needs to map the engine's color index to the variant's wire color index (and the
            // inbound `clue_type == 0 -> Color` mapping in `apply_network_action` likewise).
            Some(OutboundWebsocketMessage::GameAction {
                table_id,
                action_type,
                target: *player_index,
                value: Some(clue.clue_value as usize),
            })
        }
        GameAction::Draw { .. } => None,
    }
}
