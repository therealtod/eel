use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;

/// A technique that players agree on before the game and apply deterministically during it.
///
/// Each technique declares a priority for tier-based interpretation ordering.
pub trait ConventionTech: Sync {
    /// Priority for tier-based interpretation. Lower values = higher priority.
    fn priority(&self) -> u8;

    /// Return all actions that this technique would generate from the given POV.
    fn game_actions(&self, player_on_turn_pov: &dyn PlayerPOV) -> Vec<GameAction>;

    /// Return whether this technique explains the given action from the actor's POV.
    fn matches_action(&self, action: &GameAction, actor_pov: &dyn PlayerPOV) -> bool;

    /// Return knowledge updates that this technique implies from the given POV.
    fn knowledge_updates(&self, player_pov: &dyn PlayerPOV) -> Vec<KnowledgeUpdate>;
}
