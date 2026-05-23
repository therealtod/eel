use super::play_clue;
use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::Hypothesis;
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue to another player whose focus card is immediately playable on the stacks but
/// already gotten — the holder cannot know it's playable from the clue alone, so the clue
/// serves as a tempo提醒 ("tempo reminder") to play it now.
///
/// "Focus" follows H-Group rules: if the clue touches the receiver's chop, the chop is the
/// focus; otherwise the focus is the leftmost (newest, slot 1) newly-touched card.
///
/// A clue action is generated for every (target player, clue type, clue value) combination
/// whose focus card has a fully-known identity that is in `table_state.playable_cards()` and
/// already gotten by the receiving player.
///
/// # Limitation
/// Focus calculation uses the clue *giver's* POV to check `is_clued` on the receiver's cards.
/// The giver's knowledge does not track the receiver's convention signals, so a card in the
/// receiver's hand will be treated as unclued even if it was previously clued. This can produce
/// a wrong focus in re-clue scenarios; it is correct for freshly dealt hands.
pub struct TempoClue;

impl TempoClue {
    /// Core direct play detection: checks if the focus card is currently playable on the stacks,
    /// and it's already gotten but the holder is not aware of its playability
    fn is_tempo_clue_setup(focus_idx: CardDeckIndex, pov: &dyn PlayerPOV) -> bool {
        pov.card_identity(focus_idx).is_some_and(|card_id| {
            (pov.table_state().playable_cards(pov.static_data()) >> card_id) & 1 != 0
                && pov.is_gotten(card_id)
        })
    }
}

impl ClueTech for TempoClue {
    fn clue_game_actions(&self, active_player_pov: &dyn PlayerPOV) -> Vec<GameAction> {
        play_clue::clue_game_actions(active_player_pov, Self::is_tempo_clue_setup)
    }

    fn matches_clue(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> bool {
        play_clue::matches_clue(
            player_index,
            touched,
            clue,
            turn,
            history,
            observer_pov,
            true,
        )
    }

    fn clue_knowledge_updates(
        &self,
        player_index: PlayerIndex,
        touched: &[CardDeckIndex],
        clue: &Clue,
        turn: usize,
        history: &[GameStateSnapshot],
        observer_pov: &dyn PlayerPOV,
    ) -> Hypothesis {
        play_clue::clue_knowledge_updates(player_index, touched, clue, turn, history, observer_pov)
    }
}

impl HGroupClueTech for TempoClue {}
impl PlayClueTech for TempoClue {}
impl_convention_tech_for_hgroup_clue_tech!(TempoClue, priority::SIMPLE_PLAY_CLUE);
