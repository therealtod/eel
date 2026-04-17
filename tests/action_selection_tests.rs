mod common;

use eel::engine::action_selection_strategy::ActionSelectionStrategy;
use eel::engine::convention::hgroup::h_group_convention_set::HGroupConventionSet;
use eel::engine::convention::hgroup::tech::discard_chop::DiscardChop;
use eel::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
use eel::engine::knowledge::player_pov_view::PlayerPOVView;
use eel::engine::tree_action_selection_strategy::TreeActionSelectionStrategy;
use eel::game::action::game_action::GameAction;

fn hgroup() -> HGroupConventionSet {
    HGroupConventionSet::new(vec![
        Box::new(PlayKnownPlayable),
        Box::new(DiscardChop),
    ])
}

// ── Scenario 11: 3 players, r1 on stack, player 0 has all unknown cards ──────

// Deck index layout for scenario 11 (5 cards per player):
//   player 0 ("x" × 5):  indices 0–4
//   player 1:             indices 5–9
//   player 2:             indices 10–14

#[test]
fn discards_chop_when_no_known_playable_cards() {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(11);

    let pov = PlayerPOVView::new(
        0,
        team_knowledge.player(0),
        &team_knowledge,
        &table_state,
        &static_data,
    );

    let action = TreeActionSelectionStrategy.select_active_player_action(&pov, &hgroup());

    // Chop is the oldest unclued card in player 0's hand — deck index 0.
    assert_eq!(
        action,
        GameAction::Discard { player_index: 0, card_deck_index: 0 },
    );
}

#[test]
fn plays_card_when_its_identity_is_known_to_be_playable() {
    let (table_state, static_data, mut team_knowledge) = common::load_scenario_with_knowledge(11);

    // Narrow player 0's empathy for deck index 0 to R2 only (id = 1).
    // R1 is already on the stack in scenario 11, so R2 is playable.
    let r2_mask: u64 = 1 << 1;
    team_knowledge.player_mut(0).empathy[0] = r2_mask;

    let pov = PlayerPOVView::new(
        0,
        team_knowledge.player(0),
        &team_knowledge,
        &table_state,
        &static_data,
    );

    let action = TreeActionSelectionStrategy.select_active_player_action(&pov, &hgroup());

    assert_eq!(
        action,
        GameAction::Play { player_index: 0, card_deck_index: 0 },
    );
}

// ── Scenario 1: 4 players, all stacks empty ───────────────────────────────────

// Deck index layout for scenario 1 (4 cards per player):
//   player 0 ("x" × 4):  indices 0–3
//   player 1:             indices 4–7
//   player 2:             indices 8–11
//   player 3:             indices 12–15

#[test]
fn discards_chop_in_four_player_game_with_no_known_playable_cards() {
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(1);

    let pov = PlayerPOVView::new(
        0,
        team_knowledge.player(0),
        &team_knowledge,
        &table_state,
        &static_data,
    );

    let action = TreeActionSelectionStrategy.select_active_player_action(&pov, &hgroup());

    assert_eq!(
        action,
        GameAction::Discard { player_index: 0, card_deck_index: 0 },
    );
}

#[test]
fn selected_action_is_not_a_draw() {
    // Smoke test: the strategy must never emit a Draw action as the chosen move.
    let (table_state, static_data, team_knowledge) = common::load_scenario_with_knowledge(11);

    let pov = PlayerPOVView::new(
        0,
        team_knowledge.player(0),
        &team_knowledge,
        &table_state,
        &static_data,
    );

    let action = TreeActionSelectionStrategy.select_active_player_action(&pov, &hgroup());

    assert!(
        !matches!(action, GameAction::Draw { .. }),
        "action selection must not return a Draw; got {action:?}",
    );
}
