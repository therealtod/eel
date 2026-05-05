use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;

/// A technique that players agree on before the game and apply deterministically during it.
///
/// For clue actions, `matches_action` checks if this tech could explain the observed action.
/// The check is performed from the clue giver's POV at the time the action was performed
/// (reconstructed via `actor_pov.as_player_pov(actor_pov.player_on_turn_index())`).
pub trait ConventionTech: Sync {
    fn name(&self) -> &'static str;
    fn interpretation_priority(&self) -> u8;
    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction>;
    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool;
    fn knowledge_updates(
        &self,
        action: &GameAction,
        player_pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

// ── ClueTech ─────────────────────────────────────────────────────────────────

/// A technique that matches clue actions (gives a clue interpretation to an observed action).
///
/// When interpreting an observed action in `matches_clue` or computing `clue_knowledge_updates`,
/// the tech must check if **from the clue giver's POV at the time the action was performed** the tech could have been
/// used. Use `pov.as_player_pov(pov.player_on_turn_index())` to reconstruct the
/// clue giver's view of the game state.
pub trait ClueTech: Sync {
    fn clue_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction>;

    /// Check if this tech could explain an observed clue action.
    ///
    /// The `pov` parameter is the observer's POV. To interpret the clue correctly,
    /// reconstruct the clue giver's POV using `pov.as_player_pov(pov.player_on_turn_index())`.
    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        pov: &dyn PlayerPOV,
    ) -> bool;

    /// Compute knowledge updates for an observed clue action, from one player's perspective.
    ///
    /// Called once per player after a clue is matched to this tech. `player_index` is the clue
    /// receiver; `pov.player_on_turn_index()` is the player whose knowledge is being updated
    /// (may be the receiver or any other player that the convention affects).
    ///
    /// ## POV semantics in `knowledge_updates`
    ///
    /// By the time this method is called, `table_state.player_on_turn_index` has been set to
    /// the current observer — so `pov.player_on_turn_index()` identifies *who* is computing
    /// their knowledge, not who gave the clue.
    ///
    /// `pov`'s personal knowledge (`pov.card_identity`, etc.) comes from
    /// `team_knowledge.player(giver)` — i.e., the clue giver's empathy — because the giver
    /// has full visibility of all hands at the time the clue was given.
    ///
    /// `giver_pov(pov)` reconstructs the observer's own team-knowledge perspective
    /// (`team_knowledge.player(observer)`). Use it when the reasoning should reflect what the
    /// observer themselves knows (e.g. whether their own finesse-position card has a known
    /// identity). Use `pov` directly when the reasoning should reflect what the giver saw
    /// (e.g. confirming a card's identity from across the table).
    ///
    /// ## Typical patterns
    ///
    /// - **Clue receiver** (`pov.player_on_turn_index() == player_index`): return a
    ///   `NarrowPossibilities` update on the focus card, restricting it to identities consistent
    ///   with this tech's semantics.
    /// - **Third party** (prompted or finessed player): return `NarrowPossibilities` and/or
    ///   `AddSignal` updates on the card in their own hand that the convention targets.
    ///   Only updates for cards in the observer's `own_hand` bitmask are applied by the caller.
    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

/// Generates a `ConventionTech` impl for a type that implements `ClueTech`.
/// Usage: `impl_convention_tech_for_clue_tech!(MyType, priority_value);`
#[macro_export]
macro_rules! impl_convention_tech_for_clue_tech {
    ($t:ty, $priority:expr) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str { stringify!($t) }
            fn interpretation_priority(&self) -> u8 {
                $priority
            }

            fn game_actions(&self, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::convention_tech::ClueTech::clue_game_actions(self, pov)
            }

            fn matches_action(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> bool {
                match action.as_clue() {
                    Some((pi, touched, clue)) => $crate::engine::convention::convention_tech::ClueTech::matches_clue(self, pi, touched, clue, pov),
                    None => false,
                }
            }

            fn knowledge_updates(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                match action.as_clue() {
                    Some((pi, touched, clue)) => $crate::engine::convention::convention_tech::ClueTech::clue_knowledge_updates(self, pi, touched, clue, pov),
                    None => vec![],
                }
            }
        }
    };
}

// ── PlayTech ─────────────────────────────────────────────────────────────────

pub trait PlayTech: Sync {
    fn play_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction>;
    fn matches_play(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        pov: &dyn PlayerPOV,
    ) -> bool;
    fn play_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

#[macro_export]
macro_rules! impl_convention_tech_for_play_tech {
    ($t:ty) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str { stringify!($t) }
            fn interpretation_priority(&self) -> u8 { 0 }

            fn game_actions(&self, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::convention_tech::PlayTech::play_game_actions(self, pov)
            }

            fn matches_action(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> bool {
                if let GameAction::Play { player_index, card_deck_index } = action {
                    $crate::engine::convention::convention_tech::PlayTech::matches_play(self, *player_index, *card_deck_index, pov)
                } else {
                    false
                }
            }

            fn knowledge_updates(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                if let GameAction::Play { player_index, card_deck_index } = action {
                    $crate::engine::convention::convention_tech::PlayTech::play_knowledge_updates(self, *player_index, *card_deck_index, pov)
                } else {
                    vec![]
                }
            }
        }
    };
}

// ── DiscardTech ───────────────────────────────────────────────────────────────

pub trait DiscardTech: Sync {
    fn discard_game_actions(&self, pov: &dyn PlayerPOV) -> Vec<GameAction>;
    fn matches_discard(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        pov: &dyn PlayerPOV,
    ) -> bool;
    fn discard_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        card: CardDeckIndex,
        pov: &dyn PlayerPOV,
    ) -> Vec<KnowledgeUpdate>;
}

#[macro_export]
macro_rules! impl_convention_tech_for_discard_tech {
    ($t:ty) => {
        impl $crate::engine::convention::convention_tech::ConventionTech for $t {
            fn name(&self) -> &'static str { stringify!($t) }
            fn interpretation_priority(&self) -> u8 { 0 }

            fn game_actions(&self, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::game::action::game_action::GameAction> {
                $crate::engine::convention::convention_tech::DiscardTech::discard_game_actions(self, pov)
            }

            fn matches_action(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> bool {
                if let GameAction::Discard { player_index, card_deck_index } = action {
                    $crate::engine::convention::convention_tech::DiscardTech::matches_discard(self, *player_index, *card_deck_index, pov)
                } else {
                    false
                }
            }

            fn knowledge_updates(&self, action: &$crate::game::action::game_action::GameAction, pov: &dyn $crate::engine::knowledge::player_pov::PlayerPOV) -> Vec<$crate::engine::knowledge::knowledge_update::KnowledgeUpdate> {
                if let GameAction::Discard { player_index, card_deck_index } = action {
                    $crate::engine::convention::convention_tech::DiscardTech::discard_knowledge_updates(self, *player_index, *card_deck_index, pov)
                } else {
                    vec![]
                }
            }
        }
    };
}
