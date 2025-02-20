use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::knowledge::player_pov_view::PlayerPOVView;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::card::{CardDeckIndex, VariantCardId};
use crate::game::clue::Clue;
use crate::game::static_game_data::StaticGameData;
use crate::game::state::table_state::TableState;

/// A [TableState] with associated player knowledge and convention awareness.
///
/// This is the main integration point for the engine: it wraps a [TableState] with a
/// [TeamKnowledge] and a [ConventionSet], keeping all three in sync as actions are applied.
///
/// Two variants of each mutating method are provided:
/// - `*_of_specific_card`: used when the card identity is known (spectator / replay mode).
/// - without suffix: used when the identity is unknown (alpha-beta search over hidden state).
pub struct KnowledgeAwareGameState {
    convention_set: Box<dyn ConventionSet>,
    table_state: TableState,
    team_knowledge: TeamKnowledge,
    static_data: StaticGameData,
}

impl KnowledgeAwareGameState {
    pub fn new(convention_set: Box<dyn ConventionSet>, static_data: StaticGameData) -> Self {
        let table_state = TableState::new(&static_data);
        let team_knowledge = TeamKnowledge::new(static_data.number_of_players as usize);
        KnowledgeAwareGameState {
            convention_set,
            table_state,
            team_knowledge,
            static_data,
        }
    }

    /// Get a read-only view of the game from the specified player's perspective.
    pub fn player_pov(&self, player_index: usize) -> PlayerPOVView<'_> {
        PlayerPOVView::new(
            player_index,
            self.team_knowledge.player(player_index),
            &self.team_knowledge,
            &self.table_state,
            &self.static_data,
        )
    }

    // ── Draw ─────────────────────────────────────────────────────────────────

    /// Apply a draw, revealing the card identity to all players except the drawer.
    ///
    /// Use this in spectator / replay mode where the identity is known.
    pub fn update_with_draw_action_of_specific_card(
        &mut self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        self.table_state.update_with_draw_action(card_deck_index);
        // Delegates to TeamKnowledge, which updates teammates' empathy and the drawer's own_hand.
        self.team_knowledge.update_with_card_drawn(player_index, card_deck_index, card_id);
    }

    /// Apply a draw without revealing the card identity (e.g., during alpha-beta search).
    ///
    /// Only the drawing player's own-hand bitmask is updated; no empathy updates are made.
    pub fn update_with_draw_action(
        &mut self,
        player_index: usize,
        card_deck_index: CardDeckIndex,
    ) {
        self.table_state.update_with_draw_action(card_deck_index);
        self.team_knowledge.player_mut(player_index).own_hand |= 1 << card_deck_index;
    }

    // ── Play ─────────────────────────────────────────────────────────────────

    /// Apply a play for the current player, knowing the card's identity.
    pub fn update_with_play_action_of_specific_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let player_index = self.table_state.player_on_turn_index;
        self.table_state
            .update_with_play_action_of_specific_card(card_deck_index, card_id, &self.static_data);
        self.team_knowledge.player_mut(player_index).own_hand &= !(1u64 << card_deck_index);
    }

    /// Apply a play for the current player without knowing the card's identity.
    pub fn update_with_play_action(&mut self, card_deck_index: CardDeckIndex) {
        let player_index = self.table_state.player_on_turn_index;
        self.table_state.update_with_play_action(card_deck_index);
        self.team_knowledge.player_mut(player_index).own_hand &= !(1u64 << card_deck_index);
    }

    // ── Discard ───────────────────────────────────────────────────────────────

    /// Apply a discard for the current player, knowing the card's identity.
    pub fn update_with_discard_action_of_specific_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let player_index = self.table_state.player_on_turn_index;
        self.table_state.update_with_discard_action_of_specific_card(
            card_deck_index,
            card_id,
            &self.static_data,
        );
        self.team_knowledge.player_mut(player_index).own_hand &= !(1u64 << card_deck_index);
    }

    /// Apply a discard for the current player without knowing the card's identity.
    pub fn update_with_discard_action(&mut self, card_deck_index: CardDeckIndex) {
        let player_index = self.table_state.player_on_turn_index;
        self.table_state
            .update_with_discard_action(card_deck_index, &self.static_data);
        self.team_knowledge.player_mut(player_index).own_hand &= !(1u64 << card_deck_index);
    }

    // ── Clue ──────────────────────────────────────────────────────────────────

    /// Apply a clue action.
    ///
    /// Updates the table state (Deck empathy). Convention interpretation for knowledge
    /// updates on the receiver will be wired here once clue techs are implemented (step 5).
    pub fn update_with_clue_action(
        &mut self,
        touched_card_deck_indexes: Vec<CardDeckIndex>,
        clue: Clue,
        receiver_player_index: usize,
    ) {
        self.table_state.update_with_clue_action(
            touched_card_deck_indexes,
            clue,
            receiver_player_index,
            &self.static_data,
        );
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn table_state(&self) -> &TableState {
        &self.table_state
    }

    pub fn static_data(&self) -> &StaticGameData {
        &self.static_data
    }

    pub fn team_knowledge(&self) -> &TeamKnowledge {
        &self.team_knowledge
    }

    pub fn convention_set(&self) -> &dyn ConventionSet {
        self.convention_set.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
    use crate::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
    use crate::game::variant::test_variants::NO_VARIANT;

    fn make_state() -> KnowledgeAwareGameState {
        let static_data = StaticGameData { number_of_players: 3, variant: NO_VARIANT };
        let convention_set = Box::new(HGroupConventionSet::new(vec![Box::new(PlayKnownPlayable)]));
        KnowledgeAwareGameState::new(convention_set, static_data)
    }

    #[test]
    fn draw_with_known_identity_makes_card_visible_to_other_players() {
        let mut state = make_state();
        let card_deck_index = 5;
        let card_id = 3;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, card_id);

        // Players 1 and 2 can see the card
        assert!(state.team_knowledge().player(1).visible_cards & (1 << card_deck_index) != 0);
        assert!(state.team_knowledge().player(2).visible_cards & (1 << card_deck_index) != 0);
        // The drawer cannot see it
        assert!(state.team_knowledge().player(0).visible_cards & (1 << card_deck_index) == 0);
    }

    #[test]
    fn draw_with_known_identity_puts_card_in_drawers_own_hand() {
        let mut state = make_state();
        let card_deck_index = 5;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, 3);

        assert!(state.team_knowledge().player(0).own_hand & (1 << card_deck_index) != 0);
    }

    #[test]
    fn draw_with_known_identity_does_not_narrow_drawers_empathy() {
        let mut state = make_state();
        let card_deck_index = 5;
        let card_id = 3;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, card_id);

        // Drawer's empathy for this card should still be all possibilities
        let empathy = state.team_knowledge().player(0).empathy[card_deck_index as usize];
        assert_eq!(empathy, u64::MAX, "drawer should not know the card's identity");
    }

    #[test]
    fn play_removes_card_from_own_hand() {
        let mut state = make_state();
        let card_deck_index = 0;

        // player 0 draws then plays the card
        state.update_with_draw_action_of_specific_card(0, card_deck_index, 0);
        state.update_with_play_action_of_specific_card(card_deck_index, 0);

        assert_eq!(state.team_knowledge().player(0).own_hand & (1 << card_deck_index), 0);
    }

    #[test]
    fn discard_removes_card_from_own_hand() {
        let mut state = make_state();
        let card_deck_index = 0;

        state.update_with_draw_action_of_specific_card(0, card_deck_index, 1);
        state.update_with_discard_action_of_specific_card(card_deck_index, 1);

        assert_eq!(state.team_knowledge().player(0).own_hand & (1 << card_deck_index), 0);
    }

    #[test]
    fn player_pov_can_be_created_for_any_player() {
        let mut state = make_state();
        state.update_with_draw_action_of_specific_card(0, 7, 2);

        // Just verify player_pov builds without panic for each player.
        let _ = state.player_pov(0);
        let _ = state.player_pov(1);
        let _ = state.player_pov(2);
    }
}
