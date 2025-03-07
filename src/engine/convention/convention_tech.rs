use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;

/// A technique that players agree on before the game and apply deterministically during it.
///
/// For clue actions, `matches_action` checks if this tech could explain the observed action.
/// The check is performed from the clue giver's POV at the time the action was performed,
/// reconstructed via `history[action.turn()]`.
pub trait ConventionTech: Sync {
    fn name(&self) -> &'static str;
    fn interpretation_priority(&self) -> u8;
    fn game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction>;

    /// Check whether this tech explains the observed action.
    ///
    /// `history[turn]` is the game state captured immediately *before* the action on that turn,
    /// from which the actor's POV can be reconstructed exactly as it was at decision time.
    /// Pass `&[]` when no history has been recorded (e.g. in isolated unit tests).
    fn matches_action(
        &self,
        action: &GameAction,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool;

    /// Compute the knowledge updates produced by the action for one player.
    ///
    /// `history` is the same slice as in `matches_action`.
    fn knowledge_updates(
        &self,
        action: &GameAction,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

// ── ClueTech ─────────────────────────────────────────────────────────────────

/// A technique that matches clue actions (gives a clue interpretation to an observed action).
///
/// When interpreting an observed action in `matches_clue` or computing `clue_knowledge_updates`,
/// the tech must check if **from the clue giver's POV at the time the action was performed** the
/// tech could have been used. Use `history[turn].player_pov(giver, static_data)` to reconstruct
/// the clue giver's view of the game state.
pub trait ClueTech: Sync {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction>;

    /// Check if this tech could explain an observed clue action.
    ///
    /// `history[turn]` is the full game state captured immediately before the clue was given.
    /// Use `history[turn].player_pov(giver, static_data)` to reconstruct the exact POV the
    /// clue giver had at decision time — in particular to correctly compute the chop and focus.
    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool;

    /// Compute knowledge updates for an observed clue action, from one player's perspective.
    ///
    /// Called once per player after a clue is matched to this tech. `player_index` is the clue
    /// receiver; `pov.active_player_index()` is the player whose knowledge is being updated
    /// (may be the receiver or any other player that the convention affects).
    ///
    /// ## POV semantics in `knowledge_updates`
    ///
    /// By the time this method is called, `table_state.active_player_index` has been set to
    /// the current observer — so `pov.active_player_index()` identifies *who* is computing
    /// their knowledge, not who gave the clue.
    ///
    /// `pov`'s personal knowledge (`pov.card_identity`, etc.) comes from
    /// `team_knowledge.player(giver)` — i.e., the clue giver's empathy — because the giver
    /// has full visibility of all hands at the time the clue was given.
    ///
    /// ## Typical patterns
    ///
    /// - **Clue receiver** (`pov.active_player_index() == player_index`): return a
    ///   `NarrowPossibilities` update on the focus card, restricting it to identities consistent
    ///   with this tech's semantics.
    /// - **Third party** (prompted or finessed player): return `NarrowPossibilities` and/or
    ///   `AddSignal` updates on the card in their own hand that the convention targets.
    ///   Only updates for cards in the observer's `own_hand` bitmask are applied by the caller.
    ///
    /// `history[turn]` carries the same pre-clue game state as in `matches_clue`.  Use it to
    /// reconstruct the giver's POV at the time of the clue so that chop and focus are computed
    /// against the hand state *before* the clue touched any cards.
    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

/// Generates a `ConventionTech` impl for a type that implements `ClueTech`.
/// Usage: `impl_convention_tech_for_clue_tech!(MyType, priority_value);`
#[macro_export]
macro_rules! impl_convention_tech_for_clue_tech {
    ($t:ty, $priority:expr) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str {
                stringify!($t)
            }
            fn interpretation_priority(&self) -> u8 {
                $priority
            }

            fn game_actions(
                &self,
                active_player_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::convention_tech::ClueTech::clue_game_actions(
                    self,
                    active_player_pov,
                )
            }

            fn matches_action(
                &self,
                action: &$crate::game::action::game_action::GameAction,
                history: &[$crate::engine::game_state_snapshot::GameStateSnapshot],
                observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> bool {
                if let $crate::game::action::game_action::GameAction::Clue {
                    player_index,
                    touched_card_deck_indexes,
                    clue,
                    turn,
                } = action
                {
                    $crate::engine::convention::convention_tech::ClueTech::matches_clue(
                        self,
                        *player_index,
                        touched_card_deck_indexes,
                        clue,
                        *turn,
                        history,
                        observer_pov,
                    )
                } else {
                    false
                }
            }

            fn knowledge_updates(
                &self,
                action: &$crate::game::action::game_action::GameAction,
                history: &[$crate::engine::game_state_snapshot::GameStateSnapshot],
                observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                if let $crate::game::action::game_action::GameAction::Clue {
                    player_index,
                    touched_card_deck_indexes,
                    clue,
                    turn,
                } = action
                {
                    $crate::engine::convention::convention_tech::ClueTech::clue_knowledge_updates(
                        self,
                        *player_index,
                        touched_card_deck_indexes,
                        clue,
                        *turn,
                        history,
                        observer_pov,
                    )
                } else {
                    vec![]
                }
            }
        }
    };
}

// ── PlayTech ─────────────────────────────────────────────────────────────────

pub trait PlayTech: Sync {
    fn play_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction>;
    fn matches_play(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool;
    fn play_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

#[macro_export]
macro_rules! impl_convention_tech_for_play_tech {
    ($t:ty) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str {
                stringify!($t)
            }
            fn interpretation_priority(&self) -> u8 {
                0
            }

            fn game_actions(
                &self,
                active_player_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::convention_tech::PlayTech::play_game_actions(
                    self,
                    active_player_pov,
                )
            }

            fn matches_action(
                &self,
                action: &$crate::game::action::game_action::GameAction,
                history: &[$crate::engine::game_state_snapshot::GameStateSnapshot],
                observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> bool {
                if let GameAction::Play {
                    player_index,
                    card_deck_index,
                    turn,
                } = action
                {
                    $crate::engine::convention::convention_tech::PlayTech::matches_play(
                        self,
                        *player_index,
                        *card_deck_index,
                        *turn,
                        history,
                        observer_pov,
                    )
                } else {
                    false
                }
            }

            fn knowledge_updates(
                &self,
                action: &$crate::game::action::game_action::GameAction,
                history: &[$crate::engine::game_state_snapshot::GameStateSnapshot],
                observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV,
            ) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                if let GameAction::Play {
                    player_index,
                    card_deck_index,
                    turn,
                } = action
                {
                    $crate::engine::convention::convention_tech::PlayTech::play_knowledge_updates(
                        self,
                        *player_index,
                        *card_deck_index,
                        *turn,
                        history,
                        observer_pov,
                    )
                } else {
                    vec![]
                }
            }
        }
    };
}

// ── DiscardTech ───────────────────────────────────────────────────────────────

pub trait DiscardTech: Sync {
    fn discard_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction>;
    fn matches_discard(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool;
    fn discard_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

#[macro_export]
macro_rules! impl_convention_tech_for_discard_tech {
    ($t:ty) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str { stringify!($t) }
            fn interpretation_priority(&self) -> u8 { 0 }

            fn game_actions(&self, active_player_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::convention_tech::DiscardTech::discard_game_actions(self, active_player_pov)
            }

            fn matches_action(&self, action: &$crate::game::action::game_action::GameAction, history: &[$crate::engine::game_state_snapshot::GameStateSnapshot], observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> bool {
                if let GameAction::Discard { player_index, card_deck_index, turn } = action {
                    $crate::engine::convention::convention_tech::DiscardTech::matches_discard(self, *player_index, *card_deck_index, *turn, history, observer_pov)
                } else {
                    false
                }
            }

            fn knowledge_updates(&self, action: &$crate::game::action::game_action::GameAction, history: &[$crate::engine::game_state_snapshot::GameStateSnapshot], observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                if let GameAction::Discard { player_index, card_deck_index, turn } = action {
                    $crate::engine::convention::convention_tech::DiscardTech::discard_knowledge_updates(self, *player_index, *card_deck_index, *turn, history, observer_pov)
                } else {
                    vec![]
                }
            }
        }
    };
}
