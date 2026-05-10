use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
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

    /// Compute this tech's hypothesis for the action from the observer's POV.
    ///
    /// Each tech contributes a *single* hypothesis (its interpretation of the action).
    /// The dispatcher collects hypotheses from all matching techs into a *cohort* —
    /// the observer's effective narrowing on any card is the **union** of cohort
    /// hypothesis masks targeting that card. A hypothesis with a trigger is
    /// provisional: confirmation prunes its siblings, rejection drops the hypothesis.
    ///
    /// Return `Hypothesis::empty()` when this tech has nothing to claim from this
    /// observer's POV (typically when the action does not match the tech's action
    /// type — clue techs return empty for play/discard actions, etc.).
    fn knowledge_updates(
        &self,
        action: &GameAction,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis;
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

    /// Compute the [`Hypothesis`] this tech contributes for an observed clue action,
    /// from one observer's perspective.
    ///
    /// Called once per observer. `player_index` is the clue *receiver*;
    /// `pov.active_player_index()` identifies the observer whose knowledge is being
    /// computed (which may be the receiver, the clue giver, or any third party).
    ///
    /// ## POV semantics
    ///
    /// By the time this method is called, `table_state.active_player_index` has been
    /// set to the current observer — so `pov.active_player_index()` identifies *who*
    /// is computing their knowledge, not who gave the clue.
    ///
    /// `pov`'s personal knowledge (`pov.card_identity`, etc.) comes from
    /// `team_knowledge.player(giver)` for non-receiver observers — i.e., the clue
    /// giver's empathy — because the giver has full visibility of all hands at the
    /// time the clue was given.
    ///
    /// ## Typical patterns
    ///
    /// - **Clue receiver**: return an unconditional hypothesis with a `NarrowPossibilities`
    ///   on the focus card. If the tech is provisional (e.g. finesse waiting for a
    ///   blind-play), return a [`Hypothesis::provisional`] with a [`PendingTrigger`].
    /// - **Third party** (prompted or finessed player): return an unconditional
    ///   hypothesis with `NarrowPossibilities` and/or `AddSignal` updates on the
    ///   targeted card in the third party's own hand.
    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis;
}

/// Internal macro: expands to the `matches_action` and `knowledge_updates` method bodies shared
/// by all clue-tech `ConventionTech` impls. Not part of the public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __impl_clue_tech_matches_and_updates {
    () => {
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
        ) -> $crate::engine::knowledge::knowledge_update::Hypothesis {
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
                $crate::engine::knowledge::knowledge_update::Hypothesis::empty()
            }
        }
    };
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
            $crate::__impl_clue_tech_matches_and_updates!();
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
    ) -> Hypothesis;
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
            ) -> $crate::engine::knowledge::knowledge_update::Hypothesis {
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
                    $crate::engine::knowledge::knowledge_update::Hypothesis::empty()
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
    ) -> Hypothesis;
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

            fn knowledge_updates(&self, action: &$crate::game::action::game_action::GameAction, history: &[$crate::engine::game_state_snapshot::GameStateSnapshot], observer_pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> $crate::engine::knowledge::knowledge_update::Hypothesis {
                if let GameAction::Discard { player_index, card_deck_index, turn } = action {
                    $crate::engine::convention::convention_tech::DiscardTech::discard_knowledge_updates(self, *player_index, *card_deck_index, *turn, history, observer_pov)
                } else {
                    $crate::engine::knowledge::knowledge_update::Hypothesis::empty()
                }
            }
        }
    };
}
