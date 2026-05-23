use crate::engine::convention::convention_tech::ClueTech;
use crate::engine::convention::hgroup::h_group_core::{
    clues_for_player_with_focus, get_clue_focus,
};
use crate::engine::convention::hgroup::h_group_tech::{HGroupClueTech, PlayClueTech, priority};
use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::knowledge_update::{Hypothesis, KnowledgeUpdate};
use crate::engine::knowledge::player_pov::PlayerPOV;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::impl_convention_tech_for_hgroup_clue_tech;

/// Give a clue to another player whose focus card is immediately playable on the stacks.
///
/// "Focus" follows H-Group rules: if the clue touches the receiver's chop, the chop is the
/// focus; otherwise the focus is the leftmost (newest, slot 1) newly-touched card.
///
/// A clue action is generated for every (target player, clue type, clue value) combination
/// whose focus card has a fully-known identity that is in `table_state.playable_cards()`.
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
        let active_player_index = active_player_pov.active_player_index();
        let num_players = active_player_pov.static_data().number_of_players as usize;

        (0..num_players)
            .filter(|&x| x != active_player_index)
            .flat_map(|target| {
                clues_for_player_with_focus(target, active_player_pov)
                    .into_iter()
                    .filter_map(move |(action, focus_idx)| {
                        if Self::is_tempo_clue_setup(focus_idx, active_player_pov) {
                            Some(action)
                        } else {
                            None
                        }
                    })
            })
            .collect()
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
        let Some(game_state_snapshot) = history.get(turn.saturating_sub(1)) else {
            return false;
        };
        let giver = game_state_snapshot.table_state.active_player_index;
        let giver_pov = game_state_snapshot.player_pov(giver, observer_pov.static_data());
        let Some(focus_idx) = get_clue_focus(player_index, touched, &giver_pov) else {
            return false;
        };
        // An observer matches this tech if, from their POV, the focus card could be
        // immediately playable. Observers who see the card (Alice, Bob) have a singleton
        // empathy; the receiver (Cathy) has a wide empathy narrowed only by the clue.
        let static_data = observer_pov.static_data();
        let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
        let playable = observer_pov.table_state().playable_cards(static_data);
        let gotten = observer_pov.gotten_cards().as_bits();
        let focus_own_gotten = observer_pov
            .card_identity(focus_idx)
            .filter(|_| observer_pov.is_touched(focus_idx))
            .map(|id| 1u64 << id)
            .unwrap_or(0);
        let external_gotten = gotten & !focus_own_gotten;
        (observer_pov.inferred_identities(focus_idx).as_bits()
            & clue_mask
            & playable
            & external_gotten)
            != 0
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
        let Some(snap) = history.get(turn.saturating_sub(1)) else {
            return Hypothesis::empty();
        };
        let giver = snap.table_state.active_player_index;
        let giver_pov = snap.player_pov(giver, observer_pov.static_data());
        let focus = match get_clue_focus(player_index, touched, &giver_pov) {
            Some(f) => f,
            None => return Hypothesis::empty(),
        };
        let static_data = giver_pov.static_data();
        let total_ids =
            static_data.variant.number_of_suits as usize * static_data.variant.stacks_size as usize;
        let clue_mask = static_data.variant.empathy_for_clue(clue).as_bits();
        let mask: u64 = (0..total_ids)
            .filter(|&id| {
                if let Some(away_value) = giver_pov.away_value(id) {
                    (1u64 << id) & clue_mask != 0 && away_value == 0
                } else {
                    false
                }
            })
            .fold(0u64, |acc, id| acc | (1 << id));
        if mask == 0 {
            return Hypothesis::empty();
        }
        Hypothesis::unconditional(vec![KnowledgeUpdate::NarrowPossibilities {
            card_deck_index: focus,
            mask,
        }])
    }
}

impl HGroupClueTech for TempoClue {}
impl PlayClueTech for TempoClue {}
impl_convention_tech_for_hgroup_clue_tech!(TempoClue, priority::SIMPLE_PLAY_CLUE);
