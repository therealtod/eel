use super::*;
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::player_knowledge::PlayerKnowledge;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
use crate::game::action::game_action::GameAction;
use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::test_variants::NO_VARIANT;

/// Build a truth POV for tests by holding owned values and exposing them by reference.
/// `card_identity` falls back to the deck's public empathy via `combined_possible_identities`.
struct TruthFixture {
    knowledge: PlayerKnowledge,
    team_knowledge: TeamKnowledge,
}

impl TruthFixture {
    fn new(static_data: &StaticGameData) -> Self {
        TruthFixture {
            knowledge: PlayerKnowledge::new(0),
            team_knowledge: TeamKnowledge::new(static_data.number_of_players as usize),
        }
    }

    fn pov<'a>(
        &'a self,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
    ) -> LightweightPlayerPOV<'a> {
        LightweightPlayerPOV::with_visible_cards(
            0,
            &self.knowledge,
            &self.team_knowledge,
            table_state,
            static_data,
            u64::MAX,
        )
    }
}

fn make_state(strikes: u8) -> (TableState, StaticGameData) {
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        Deck::new(&NO_VARIANT),
        Hand::empty_array(),
        0,
        0,
        PlayingStacks::empty(),
        strikes,
        CopiesCountingCardCollection::empty(),
    );
    (state, static_data)
}

fn make_kags(strikes: u8) -> KnowledgeAwareGameState {
    let (table_state, static_data) = make_state(strikes);
    let tk = TeamKnowledge::new(static_data.number_of_players as usize);
    KnowledgeAwareGameState::from_parts(static_data, table_state, tk, 0)
}

#[test]
fn higher_strikes_produce_lower_score() {
    let evaluator = DefaultEvaluator::default();
    let s0 = make_kags(0);
    let s1 = make_kags(1);
    let s2 = make_kags(2);
    let truth = TruthFixture::new(s0.static_data());
    let p0 = truth.pov(s0.table_state(), s0.static_data());
    let p1 = truth.pov(s1.table_state(), s1.static_data());
    let p2 = truth.pov(s2.table_state(), s2.static_data());
    assert!(evaluator.score(&s0, &p0, 0) > evaluator.score(&s1, &p1, 0));
    assert!(evaluator.score(&s1, &p1, 0) > evaluator.score(&s2, &p2, 0));
}

#[test]
fn signal_ignored_penalty_fires_when_actor_skips_signalled_card() {
    use crate::engine::convention::hgroup::signal::Signal;
    let evaluator = DefaultEvaluator::default();
    let (mut table_state, static_data) = make_state(0);
    table_state.hands[0] = Hand::new(&[5]);
    let mut tk = TeamKnowledge::new(3);
    tk.player_mut(0).own_hand = 1 << 5;
    tk.player_mut(0).signals[5].push(Signal::Play {
        card_deck_index: 5,
        committed_identity: 0,
    });

    let discard = GameAction::Discard {
        player_index: 0,
        card_deck_index: 5,
        turn: 1,
    };
    let pen = evaluator.signal_ignored_penalty(&discard, 0, &static_data, &tk, &table_state);
    assert_eq!(pen, -evaluator.signal_ignored_penalty_weight);

    let clue = GameAction::Clue {
        player_index: 1,
        touched_card_deck_indexes: smallvec::smallvec![],
        clue: crate::game::clue::Clue {
            clue_type: crate::game::clue_type::ClueType::Rank,
            clue_value: 1,
        },
        turn: 1,
    };
    let pen = evaluator.signal_ignored_penalty(&clue, 0, &static_data, &tk, &table_state);
    assert_eq!(pen, -evaluator.signal_ignored_penalty_weight);

    let play = GameAction::Play {
        player_index: 0,
        card_deck_index: 5,
        turn: 1,
    };
    let pen = evaluator.signal_ignored_penalty(&play, 0, &static_data, &tk, &table_state);
    assert_eq!(pen, 0.0);
}

#[test]
fn signal_ignored_penalty_zero_when_no_active_signal() {
    let evaluator = DefaultEvaluator::default();
    let (mut table_state, static_data) = make_state(0);
    table_state.hands[0] = Hand::new(&[5]);
    let mut tk = TeamKnowledge::new(3);
    tk.player_mut(0).own_hand = 1 << 5;

    let discard = GameAction::Discard {
        player_index: 0,
        card_deck_index: 5,
        turn: 1,
    };
    let pen = evaluator.signal_ignored_penalty(&discard, 0, &static_data, &tk, &table_state);
    assert_eq!(pen, 0.0);
}

#[test]
fn signal_ignored_penalty_ignores_touched_signalled_cards() {
    use crate::engine::convention::hgroup::signal::Signal;
    let evaluator = DefaultEvaluator::default();
    let (mut table_state, static_data) = make_state(0);
    table_state.hands[0] = Hand::new(&[5]);
    table_state.clue_touched_cards = 1 << 5;
    let mut tk = TeamKnowledge::new(3);
    tk.player_mut(0).own_hand = 1 << 5;
    tk.player_mut(0).signals[5].push(Signal::Play {
        card_deck_index: 5,
        committed_identity: 0,
    });

    let discard = GameAction::Discard {
        player_index: 0,
        card_deck_index: 5,
        turn: 1,
    };
    let pen = evaluator.signal_ignored_penalty(&discard, 0, &static_data, &tk, &table_state);
    assert_eq!(pen, 0.0);
}

#[test]
fn misinformation_term_fires_on_excluded_truth() {
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;

    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };

    let mut deck = Deck::new(&NO_VARIANT);
    deck.reveal_card(5, 7, &NO_VARIANT);

    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        Hand::empty_array(),
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );

    let mut tk = TeamKnowledge::new(3);
    let mut pk0 = knowledge_for_hand(&[5]);
    pk0.inferred_identities[5] = Some(crate::game::card::CardIdentityMask::from_bits(1 << 3));
    *tk.player_mut(0) = pk0;

    let truth_fixture = TruthFixture::new(&static_data);
    let truth = truth_fixture.pov(&state, &static_data);
    let score_misinformed =
        DefaultEvaluator::misinformation_score(&static_data, &tk, &state, &truth);
    assert!(
        score_misinformed > 0.0,
        "misinformation_score should be positive when effective mask excludes truth (got {score_misinformed})"
    );

    tk.player_mut(0).inferred_identities[5] =
        Some(crate::game::card::CardIdentityMask::from_bits(1 << 7));
    let score_correct = DefaultEvaluator::misinformation_score(&static_data, &tk, &state, &truth);
    assert_eq!(
        score_correct, 0.0,
        "misinformation_score should be 0 when knowledge exactly matches truth"
    );
}

fn make_chop_test_state(ids_in_hand: [u8; 5]) -> (TableState, StaticGameData, TeamKnowledge) {
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    for (slot, &id) in ids_in_hand.iter().enumerate() {
        deck.reveal_card(slot as u8, id as usize, &NO_VARIANT);
    }
    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[0, 1, 2, 3, 4]);
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let mut tk = TeamKnowledge::new(3);
    *tk.player_mut(0) = knowledge_for_hand(&[0, 1, 2, 3, 4]);
    (state, static_data, tk)
}

#[test]
fn critical_exposure_chop_no_buffer_is_one() {
    let (state, static_data, tk) = make_chop_test_state([4, 7, 12, 17, 22]);
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score = DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    assert!(
        (score - 1.0).abs() < 1e-9,
        "chop critical with no buffer should be exactly 1.0, got {score}"
    );
}

#[test]
fn critical_exposure_slot1_lower_than_chop() {
    let (state, static_data, tk) = make_chop_test_state([7, 12, 17, 22, 4]);
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score = DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    let expected = 1.0 / (1.0 + 4.0);
    assert!(
        (score - expected).abs() < 1e-9,
        "slot-1 critical with no buffer should be {expected}, got {score}"
    );
}

#[test]
fn critical_exposure_buffer_dominates_position() {
    let (mut state, static_data, tk) = make_chop_test_state([4, 0, 5, 10, 15]);
    let mut tk = tk;
    for (slot, id) in [(1, 0), (2, 5), (3, 10), (4, 15)] {
        tk.player_mut(0).inferred_identities[slot] =
            Some(crate::game::card::CardIdentityMask::from_bits(1 << id));
        state.clue_touched_cards |= 1u64 << slot;
    }
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score = DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    let expected = 1.0 / (1.0 + 0.0 + 4.0 * 4.0);
    assert!(
        (score - expected).abs() < 1e-9,
        "chop critical with 4 buffer should be {expected}, got {score}"
    );
}

#[test]
fn critical_exposure_zero_when_no_critical_mask() {
    let (state, static_data, tk) = make_chop_test_state([0, 5, 10, 15, 20]);
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score = DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    assert_eq!(score, 0.0, "no critical identities → zero exposure");
}

#[test]
fn critical_exposure_zero_when_critical_is_touched() {
    let (mut state, static_data, tk) = make_chop_test_state([4, 7, 12, 17, 22]);
    state.clue_touched_cards |= 1u64 << 0;
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score = DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    assert_eq!(
        score, 0.0,
        "touched non-playable critical should not contribute to exposure"
    );
}

#[test]
fn bdr_full_deck_is_small() {
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let deck = Deck::new(&NO_VARIANT);
    let hands = Hand::empty_array();
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let bdr = DefaultEvaluator::bottom_deck_risk_score(0, &state, &static_data, &truth_pov);
    assert!(
        bdr < 0.5,
        "full-deck BDR should be small (got {bdr}); Term B = 1/50 should dominate"
    );
}

#[test]
fn bdr_empty_deck_is_large() {
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    deck.current_size = 1;
    let hands = Hand::empty_array();
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let bdr = DefaultEvaluator::bottom_deck_risk_score(0, &state, &static_data, &truth_pov);
    assert!(
        bdr > 1.0,
        "near-empty-deck BDR should be large (got {bdr}); Term B = 1/1 = 1.0"
    );
}

#[test]
fn bdr_zero_when_surviving_copy_visible() {
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    deck.reveal_card(0, 1, &NO_VARIANT);
    deck.reveal_card(1, 1, &NO_VARIANT);
    let mut hands = Hand::empty_array();
    hands[1] = Hand::new(&[1]);
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let bdr = DefaultEvaluator::bottom_deck_risk_score(0, &state, &static_data, &truth_pov);
    assert_eq!(bdr, 0.0, "visible surviving copy must zero out BDR");
}

#[test]
fn bdr_scales_linearly_with_weight() {
    use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let deck = Deck::new(&NO_VARIANT);
    let hands = Hand::empty_array();
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let tk = TeamKnowledge::new(static_data.number_of_players as usize);
    let kags = KnowledgeAwareGameState::from_parts(static_data, state, tk, 0);
    let truth = TruthFixture::new(kags.static_data());
    let truth_pov = truth.pov(kags.table_state(), kags.static_data());
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let w1 = DefaultEvaluator {
        bottom_deck_risk_weight: 0.1,
        ..DefaultEvaluator::default()
    };
    let w2 = DefaultEvaluator {
        bottom_deck_risk_weight: 0.5,
        ..DefaultEvaluator::default()
    };
    let w1 = DefaultEvaluator {
        discard_while_known_playable_penalty: 0.0,
        ..w1
    };
    let w2 = DefaultEvaluator {
        discard_while_known_playable_penalty: 0.0,
        ..w2
    };
    let p1 = w1.discard_action_penalty(&action, 0, &kags, &truth_pov);
    let p2 = w2.discard_action_penalty(&action, 0, &kags, &truth_pov);
    assert!(p1 < 0.0 && p2 < 0.0, "BDR penalty must be negative");
    let ratio = p2 / p1;
    let expected = 0.5 / 0.1;
    assert!(
        (ratio - expected).abs() < 1e-9,
        "penalty ratio {ratio} should equal weight ratio {expected}"
    );
}

#[test]
fn critical_exposure_zero_for_unresolved_chop() {
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    for (offset, &id) in [7u8, 12, 17, 22].iter().enumerate() {
        deck.reveal_card((offset + 1) as u8, id as usize, &NO_VARIANT);
    }
    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[0, 1, 2, 3, 4]);
    let state = TableState::from_parts(
        ClueTokenBank::new(10),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let mut tk = TeamKnowledge::new(3);
    *tk.player_mut(0) = knowledge_for_hand(&[0, 1, 2, 3, 4]);
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score = DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    assert_eq!(
        score, 0.0,
        "unresolved chop (no truth identity) must not contribute to critical exposure"
    );
}

fn make_critical_chop_states(
    with_clue_tokens: bool,
) -> (
    KnowledgeAwareGameState,
    KnowledgeAwareGameState,
    KnowledgeAwareGameState,
) {
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
    use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;

    let static_data = StaticGameData {
        number_of_players: 2,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    deck.reveal_card(10, 4, &NO_VARIANT);
    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[0]);
    hands[1] = Hand::new(&[10]);
    let clue_tokens = if with_clue_tokens { 8 } else { 0 };
    let pre_state = TableState::from_parts(
        ClueTokenBank::new(clue_tokens),
        deck.clone(),
        hands.clone(),
        0,
        1,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let mut tk = TeamKnowledge::new(2);
    *tk.player_mut(0) = knowledge_for_hand(&[0]);
    *tk.player_mut(1) = knowledge_for_hand(&[10]);
    let pre =
        KnowledgeAwareGameState::from_parts(static_data.clone(), pre_state.clone(), tk.clone(), 11);

    let post_unsaved =
        KnowledgeAwareGameState::from_parts(static_data.clone(), pre_state.clone(), tk.clone(), 11);

    let mut saved_state = pre_state.clone();
    saved_state.clue_touched_cards |= 1u64 << 10;
    let post_saved = KnowledgeAwareGameState::from_parts(static_data, saved_state, tk, 11);

    (pre, post_unsaved, post_saved)
}

#[test]
fn critical_exposure_delta_penalty_when_save_skipped() {
    let (pre, post_unsaved, _) = make_critical_chop_states(true);
    let evaluator = DefaultEvaluator {
        critical_exposure_delta_weight: 2.0,
        ..DefaultEvaluator::default()
    };
    let truth = TruthFixture::new(pre.static_data());
    let truth_pov = truth.pov(pre.table_state(), pre.static_data());
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let delta =
        evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_unsaved, &truth_pov);
    assert!(
        delta < 0.0,
        "should charge when critical chop left unsaved with tokens available (got {delta})"
    );
    assert!(
        (delta - (-2.0)).abs() < 1e-9,
        "penalty should equal -weight (got {delta})"
    );
}

#[test]
fn critical_exposure_delta_reward_when_chop_saved() {
    let (pre, _, post_saved) = make_critical_chop_states(true);
    let evaluator = DefaultEvaluator {
        critical_exposure_delta_weight: 2.0,
        ..DefaultEvaluator::default()
    };
    let truth = TruthFixture::new(pre.static_data());
    let truth_pov = truth.pov(pre.table_state(), pre.static_data());
    let action = GameAction::Clue {
        player_index: 1,
        touched_card_deck_indexes: smallvec::smallvec![10],
        clue: crate::game::clue::Clue {
            clue_type: crate::game::clue_type::ClueType::Rank,
            clue_value: 5,
        },
        turn: 1,
    };
    let delta = evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_saved, &truth_pov);
    assert!(
        delta > 0.0,
        "should reward when critical chop card is saved (got {delta})"
    );
    assert!(
        (delta - 2.0).abs() < 1e-9,
        "reward should equal +weight (got {delta})"
    );
}

#[test]
fn critical_exposure_delta_no_penalty_when_no_clue_tokens() {
    let (pre, post_unsaved, _) = make_critical_chop_states(false);
    let evaluator = DefaultEvaluator {
        critical_exposure_delta_weight: 2.0,
        ..DefaultEvaluator::default()
    };
    let truth = TruthFixture::new(pre.static_data());
    let truth_pov = truth.pov(pre.table_state(), pre.static_data());
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let delta =
        evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_unsaved, &truth_pov);
    assert_eq!(
        delta, 0.0,
        "no penalty when no clue tokens available (got {delta})"
    );
}

#[test]
fn critical_exposure_delta_zero_when_no_critical_cards() {
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;
    use crate::engine::knowledge_aware_game_state::KnowledgeAwareGameState;

    let static_data = StaticGameData {
        number_of_players: 2,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    deck.reveal_card(10, 0, &NO_VARIANT);
    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[0]);
    hands[1] = Hand::new(&[10]);
    let state = TableState::from_parts(
        ClueTokenBank::new(8),
        deck,
        hands,
        0,
        1,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let mut tk = TeamKnowledge::new(2);
    *tk.player_mut(0) = knowledge_for_hand(&[0]);
    *tk.player_mut(1) = knowledge_for_hand(&[10]);
    let kags = KnowledgeAwareGameState::from_parts(static_data.clone(), state.clone(), tk, 11);
    let evaluator = DefaultEvaluator {
        critical_exposure_delta_weight: 2.0,
        ..DefaultEvaluator::default()
    };
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let delta = evaluator.critical_exposure_delta_bonus(&action, 0, &kags, &kags, &truth_pov);
    assert_eq!(
        delta, 0.0,
        "no effect when no critical cards exist (got {delta})"
    );
}

#[test]
fn critical_exposure_delta_scales_with_buffer_depth() {
    use crate::engine::convention::hgroup::signal::Signal;
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;

    let static_data = StaticGameData {
        number_of_players: 2,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    deck.reveal_card(10, 4, &NO_VARIANT);
    deck.reveal_card(11, 0, &NO_VARIANT);
    deck.reveal_card(12, 5, &NO_VARIANT);
    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[0]);
    hands[1] = Hand::new(&[10, 11, 12]);
    let pre_state = TableState::from_parts(
        ClueTokenBank::new(8),
        deck,
        hands,
        0,
        1,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );
    let mut tk = TeamKnowledge::new(2);
    *tk.player_mut(0) = knowledge_for_hand(&[0]);
    let mut pk1 = knowledge_for_hand(&[10, 11, 12]);
    pk1.signals[11].push(Signal::Play {
        card_deck_index: 11,
        committed_identity: 0,
    });
    pk1.signals[12].push(Signal::Play {
        card_deck_index: 12,
        committed_identity: 5,
    });
    *tk.player_mut(1) = pk1;

    let pre =
        KnowledgeAwareGameState::from_parts(static_data.clone(), pre_state.clone(), tk.clone(), 13);
    let post_unsaved = pre.clone();
    let mut saved_state = pre_state.clone();
    saved_state.clue_touched_cards |= 1u64 << 10;
    let post_saved = KnowledgeAwareGameState::from_parts(static_data.clone(), saved_state, tk, 13);

    let weight = 2.0_f64;
    let evaluator = DefaultEvaluator {
        critical_exposure_delta_weight: weight,
        ..DefaultEvaluator::default()
    };
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(pre.table_state(), &static_data);
    let discard = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let save_clue = GameAction::Clue {
        player_index: 1,
        touched_card_deck_indexes: smallvec::smallvec![10],
        clue: crate::game::clue::Clue {
            clue_type: crate::game::clue_type::ClueType::Rank,
            clue_value: 5,
        },
        turn: 1,
    };

    let expected_threat = 1.0 / (1.0 + 4.0 * 2.0);

    let penalty =
        evaluator.critical_exposure_delta_bonus(&discard, 0, &pre, &post_unsaved, &truth_pov);
    assert!(
        (penalty - (-weight * expected_threat)).abs() < 1e-9,
        "penalty should be -weight*threat = {:.6} with buffer=2, got {penalty:.6}",
        -weight * expected_threat
    );

    let reward =
        evaluator.critical_exposure_delta_bonus(&save_clue, 0, &pre, &post_saved, &truth_pov);
    assert!(
        (reward - (weight * expected_threat)).abs() < 1e-9,
        "reward should be +weight*threat = {:.6} with buffer=2, got {reward:.6}",
        weight * expected_threat
    );
}

fn make_kags_with_stacks(score: u8) -> KnowledgeAwareGameState {
    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let stacks = if score == 0 {
        PlayingStacks::empty()
    } else {
        let cards: Vec<usize> = (0..score as usize).map(|s| s * 5).collect();
        PlayingStacks::new(cards, &NO_VARIANT)
    };
    let state = TableState::from_parts(
        ClueTokenBank::new(8),
        Deck::new(&NO_VARIANT),
        Hand::empty_array(),
        0,
        0,
        stacks,
        0,
        CopiesCountingCardCollection::empty(),
    );
    let tk = TeamKnowledge::new(static_data.number_of_players as usize);
    KnowledgeAwareGameState::from_parts(static_data, state, tk, 0)
}

#[test]
fn tempo_cost_zero_for_successful_play() {
    let evaluator = DefaultEvaluator::default();
    let pre = make_kags_with_stacks(0);
    let post = make_kags_with_stacks(1);
    let action = GameAction::Play {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let cost = evaluator.tempo_cost(&action, &pre, &post, 0, 0);
    assert_eq!(cost, 0.0, "successful play must have zero tempo cost");
}

#[test]
fn tempo_cost_charges_misplay() {
    let evaluator = DefaultEvaluator::default();
    let pre = make_kags_with_stacks(0);
    let mut post = make_kags_with_stacks(0);
    post.table_state.strike_tokens = 1;
    let action = GameAction::Play {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let cost = evaluator.tempo_cost(&action, &pre, &post, 0, 0);
    assert_eq!(
        cost, -evaluator.tempo_cost_weight,
        "misplay (strike incremented) must pay tempo cost"
    );
}

#[test]
fn tempo_cost_zero_for_vanish_play() {
    let evaluator = DefaultEvaluator::default();
    let pre = make_kags_with_stacks(0);
    let post = make_kags_with_stacks(0);
    let action = GameAction::Play {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let cost = evaluator.tempo_cost(&action, &pre, &post, 0, 0);
    assert_eq!(
        cost, 0.0,
        "vanish play (no score advance and no strike) must be tempo neutral"
    );
}

#[test]
fn tempo_cost_charges_clue() {
    let evaluator = DefaultEvaluator::default();
    let state = make_kags_with_stacks(0);
    let action = GameAction::Clue {
        player_index: 1,
        touched_card_deck_indexes: smallvec::smallvec![],
        clue: crate::game::clue::Clue {
            clue_type: crate::game::clue_type::ClueType::Rank,
            clue_value: 1,
        },
        turn: 1,
    };
    let cost = evaluator.tempo_cost(&action, &state, &state, 0, 0);
    assert_eq!(
        cost, -evaluator.tempo_cost_weight,
        "clue must pay tempo cost"
    );
}

#[test]
fn tempo_cost_does_not_charge_discard() {
    let evaluator = DefaultEvaluator::default();
    let state = make_kags_with_stacks(0);
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let cost = evaluator.tempo_cost(&action, &state, &state, 0, 0);
    assert_eq!(cost, 0.0, "discard must be tempo neutral");
}

#[test]
fn tempo_cost_respects_weight_zero() {
    let evaluator = DefaultEvaluator {
        tempo_cost_weight: 0.0,
        ..DefaultEvaluator::default()
    };
    let state = make_kags_with_stacks(0);
    let discard = GameAction::Discard {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let clue = GameAction::Clue {
        player_index: 1,
        touched_card_deck_indexes: smallvec::smallvec![],
        clue: crate::game::clue::Clue {
            clue_type: crate::game::clue_type::ClueType::Rank,
            clue_value: 1,
        },
        turn: 1,
    };
    assert_eq!(evaluator.tempo_cost(&discard, &state, &state, 0, 0), 0.0);
    assert_eq!(evaluator.tempo_cost(&clue, &state, &state, 0, 0), 0.0);
}

// ---- BDR POV-identity tests -----------------------------------------------
//
// bottom_deck_risk_score must derive its empathy from the evaluating POV's
// inferred_identities, not the deck's global (truth-narrowed) empathy.  The
// two tests below place card 5 in player 0's own hand with different
// inferred-identity masks and verify the resulting BDR.

/// Helper: POV fixture where the player cannot see their own hand
/// (visible_cards = 0).  `knowledge` is taken by value so callers can set
/// own_hand and inferred_identities before passing it in.
struct OwnHandPOV {
    knowledge: PlayerKnowledge,
    team_knowledge: TeamKnowledge,
}

impl OwnHandPOV {
    fn new(knowledge: PlayerKnowledge, static_data: &StaticGameData) -> Self {
        OwnHandPOV {
            knowledge,
            team_knowledge: TeamKnowledge::new(static_data.number_of_players as usize),
        }
    }

    fn pov<'a>(
        &'a self,
        table_state: &'a TableState,
        static_data: &'a StaticGameData,
    ) -> LightweightPlayerPOV<'a> {
        LightweightPlayerPOV::with_visible_cards(
            0,
            &self.knowledge,
            &self.team_knowledge,
            table_state,
            static_data,
            0, // player cannot see their own hand
        )
    }
}

#[test]
fn bdr_zero_when_pov_inferred_identities_excludes_all_bdr_cards() {
    // Card 5 is globally revealed as R2 (id 1, 2 copies → BDR would trigger).
    // The evaluating POV's inferred_identities for card 5 contains only rank-1
    // identities (3 copies each), which can never drive remaining copies to 1.
    // The new code must use the POV mask → BDR = 0.
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;

    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    // Reveal card 5 as R2 (identity 1): global empathy = single bit for id 1.
    deck.reveal_card(5, 1, &NO_VARIANT);

    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[5]);
    let state = TableState::from_parts(
        ClueTokenBank::new(8),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );

    // POV: player 0 holds card 5 but doesn't know its identity.
    // Inferred mask = all rank-1 ids (3 copies → BDR can't trigger).
    let rank1_mask: u64 = (1 << 0) | (1 << 5) | (1 << 10) | (1 << 15) | (1 << 20);
    let mut pk = knowledge_for_hand(&[5]);
    pk.inferred_identities[5] = Some(crate::game::card::CardIdentityMask::from_bits(rank1_mask));
    let fixture = OwnHandPOV::new(pk, &static_data);
    let pov = fixture.pov(&state, &static_data);

    let bdr = DefaultEvaluator::bottom_deck_risk_score(5, &state, &static_data, &pov);
    assert_eq!(
        bdr, 0.0,
        "BDR must be 0 when POV inferred_identities contains only 3-copy cards; \
         the deck's truth (R2) must not leak into the POV's risk assessment (got {bdr})"
    );
}

#[test]
fn bdr_positive_when_pov_inferred_identities_are_two_copy_cards() {
    // Card 5 is globally revealed as R1 (id 0, 3 copies → BDR doesn't trigger
    // from global empathy).  The evaluating POV's inferred_identities contains
    // only rank-2 ids (2 copies → BDR does trigger).
    // The new code must use the POV mask → BDR > 0.
    use crate::engine::knowledge::player_knowledge::knowledge_for_hand;

    let static_data = StaticGameData {
        number_of_players: 3,
        variant: NO_VARIANT,
    };
    let mut deck = Deck::new(&NO_VARIANT);
    // Reveal card 5 as R1 (identity 0): global empathy = single bit for id 0.
    deck.reveal_card(5, 0, &NO_VARIANT);

    let mut hands = Hand::empty_array();
    hands[0] = Hand::new(&[5]);
    let state = TableState::from_parts(
        ClueTokenBank::new(8),
        deck,
        hands,
        0,
        0,
        PlayingStacks::empty(),
        0,
        CopiesCountingCardCollection::empty(),
    );

    // POV: inferred mask = all rank-2 ids (2 copies → BDR triggers for each).
    let rank2_mask: u64 = (1 << 1) | (1 << 6) | (1 << 11) | (1 << 16) | (1 << 21);
    let mut pk = knowledge_for_hand(&[5]);
    pk.inferred_identities[5] = Some(crate::game::card::CardIdentityMask::from_bits(rank2_mask));
    let fixture = OwnHandPOV::new(pk, &static_data);
    let pov = fixture.pov(&state, &static_data);

    let bdr = DefaultEvaluator::bottom_deck_risk_score(5, &state, &static_data, &pov);
    assert!(
        bdr > 0.0,
        "BDR must be positive when POV inferred_identities contains only 2-copy cards; \
         the deck's truth (R1, 3 copies) must not suppress the POV's risk assessment (got {bdr})"
    );
}

// ── Save Principle: forced-save bad-touch discount ──────────────────────────

/// Build a 3-player position where player 1's chop is a critical R4 and an interior card is a
/// trash R1 (already on the red stack). A red colour clue saves the critical chop but is forced
/// to bad-touch the trash R1; a rank-1 clue touches only the trash and saves nothing.
fn forced_save_position() -> (TableState, StaticGameData) {
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
    let (mut table_state, static_data) = make_state(0);

    // Play R1 to the red stack so a second R1 becomes trash.
    table_state.active_player_index = 0;
    table_state.update_with_draw_action(0);
    table_state.update_with_play_action_of_specific_card(
        0,
        NoVarCards::R1.as_variant_card_id(),
        &static_data,
    );

    // Player 1: chop = R4 (oldest), interior = trash R1 (newer).
    table_state.active_player_index = 1;
    table_state.update_with_draw_action(10); // chop, R4
    table_state.update_with_draw_action(11); // newer, R1 (trash)
    table_state.active_player_index = 0;
    table_state
        .deck
        .reveal_card(10, NoVarCards::R4.as_variant_card_id(), &NO_VARIANT);
    table_state
        .deck
        .reveal_card(11, NoVarCards::R1.as_variant_card_id(), &NO_VARIANT);
    // Discard one R4 → the chop R4 is now critical.
    table_state
        .discard_pile
        .add_card_with_id(NoVarCards::R4.as_variant_card_id());

    (table_state, static_data)
}

#[test]
fn forced_save_bad_touch_is_discounted() {
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;

    let (table_state, static_data) = forced_save_position();
    let tk = TeamKnowledge::new(static_data.number_of_players as usize);
    let truth = TruthFixture::new(&static_data);
    let pov = truth.pov(&table_state, &static_data);

    // Red colour clue saves the critical chop (deck 10) but also touches trash R1 (deck 11).
    let clue = Clue {
        clue_type: ClueType::Color,
        clue_value: 0,
    };
    let touched = [10u8, 11];

    let discounted = DefaultEvaluator::default();
    // Control evaluator: same in every way except the forced-save relief is disabled (0), so the
    // full good-touch penalty applies even to a forced save.
    let full = DefaultEvaluator {
        forced_save_bad_touch_relief: 0.0,
        ..DefaultEvaluator::default()
    };

    let b_discounted = discounted.clue_precision_bonus(
        &clue,
        &touched,
        1,
        0,
        &pov,
        &static_data,
        &tk,
        &table_state,
    );
    let b_full =
        full.clue_precision_bonus(&clue, &touched, 1, 0, &pov, &static_data, &tk, &table_state);

    // Exactly one forced bad touch (the trash R1), so the relief saves one
    // `forced_save_bad_touch_relief` worth of penalty.
    assert!(
        b_discounted > b_full,
        "forced save should be penalised less than a deliberate bad touch \
         (discounted={b_discounted}, full={b_full})"
    );
    let expected_gap = discounted.forced_save_bad_touch_relief;
    assert!(
        (b_discounted - b_full - expected_gap).abs() < 1e-9,
        "discount gap should equal one forced-save relief unit; got {}",
        b_discounted - b_full
    );
}

#[test]
fn non_save_bad_touch_uses_full_penalty() {
    use crate::game::clue::Clue;
    use crate::game::clue_type::ClueType;

    let (table_state, static_data) = forced_save_position();
    let tk = TeamKnowledge::new(static_data.number_of_players as usize);
    let truth = TruthFixture::new(&static_data);
    let pov = truth.pov(&table_state, &static_data);

    // Rank-1 clue touches only the trash R1 (deck 11); it does NOT touch the critical chop,
    // so it is not a forced save and must pay the full good-touch penalty.
    let clue = Clue {
        clue_type: ClueType::Rank,
        clue_value: 1,
    };
    let touched = [11u8];

    let discounted = DefaultEvaluator::default();
    let full = DefaultEvaluator {
        forced_save_bad_touch_relief: 0.0,
        ..DefaultEvaluator::default()
    };

    let b_discounted = discounted.clue_precision_bonus(
        &clue,
        &touched,
        1,
        0,
        &pov,
        &static_data,
        &tk,
        &table_state,
    );
    let b_full =
        full.clue_precision_bonus(&clue, &touched, 1, 0, &pov, &static_data, &tk, &table_state);

    assert!(
        (b_discounted - b_full).abs() < 1e-9,
        "a non-save bad touch must be charged the full penalty regardless of the forced-save \
         discount (discounted={b_discounted}, full={b_full})"
    );
}

// ── Save Principle: known-playable discard relief under save pressure ────────

/// 3-player position where the actor (player 0) holds a known-playable (a `Signal::Play` on deck
/// 5) and player 1's unclued chop (deck 10) is an R4. When `critical` is true one R4 is discarded,
/// making the chop a critical Save-Principle-protected card in concrete danger (player 1 has no
/// known-playable of their own); when false the chop is an uncritical, unprotected R4.
fn save_pressure_position(critical: bool) -> KnowledgeAwareGameState {
    use crate::engine::convention::hgroup::signal::Signal;
    use crate::game::deck::unit_test_constants::novariant_constants::NoVarCards;
    let (mut table_state, static_data) = make_state(0);
    table_state.hands[0] = Hand::new(&[5]);
    table_state.hands[1] = Hand::new(&[10]);
    table_state
        .deck
        .reveal_card(10, NoVarCards::R4.as_variant_card_id(), &NO_VARIANT);
    if critical {
        // One R4 discarded -> the chop R4 is the last live copy -> critical.
        table_state
            .discard_pile
            .add_card_with_id(NoVarCards::R4.as_variant_card_id());
    }
    let mut tk = TeamKnowledge::new(static_data.number_of_players as usize);
    tk.player_mut(0).own_hand = 1 << 5;
    tk.player_mut(0).signals[5].push(Signal::Play {
        card_deck_index: 5,
        committed_identity: NoVarCards::R1.as_variant_card_id(),
    });
    KnowledgeAwareGameState::from_parts(static_data, table_state, tk, 0)
}

#[test]
fn discard_while_known_playable_softened_under_save_pressure() {
    let kags = save_pressure_position(true);
    let truth = TruthFixture::new(kags.static_data());
    let truth_pov = truth.pov(kags.table_state(), kags.static_data());
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 5,
        turn: 1,
    };

    // Relief disabled (full penalty) vs default (relief = 5). BDR is identical between the two, so
    // the only difference is the softened known-playable penalty.
    let full = DefaultEvaluator {
        save_pressure_known_playable_relief: 0.0,
        ..DefaultEvaluator::default()
    };
    let relieved = DefaultEvaluator::default();

    let p_full = full.discard_action_penalty(&action, 0, &kags, &truth_pov);
    let p_relieved = relieved.discard_action_penalty(&action, 0, &kags, &truth_pov);

    assert!(
        p_relieved > p_full,
        "discarding while holding a known-playable should be softened under save pressure \
         (relieved={p_relieved}, full={p_full})"
    );
    assert!(
        (p_relieved - p_full - relieved.save_pressure_known_playable_relief).abs() < 1e-9,
        "relief gap should equal one save_pressure_known_playable_relief unit; got {}",
        p_relieved - p_full
    );
}

#[test]
fn discard_while_known_playable_full_penalty_without_save_pressure() {
    // Same shape, but player 1's chop R4 is not critical and not otherwise protected, so there is
    // no save pressure and the relief must not apply.
    let kags = save_pressure_position(false);
    let truth = TruthFixture::new(kags.static_data());
    let truth_pov = truth.pov(kags.table_state(), kags.static_data());
    let action = GameAction::Discard {
        player_index: 0,
        card_deck_index: 5,
        turn: 1,
    };

    let full = DefaultEvaluator {
        save_pressure_known_playable_relief: 0.0,
        ..DefaultEvaluator::default()
    };
    let relieved = DefaultEvaluator::default();

    let p_full = full.discard_action_penalty(&action, 0, &kags, &truth_pov);
    let p_relieved = relieved.discard_action_penalty(&action, 0, &kags, &truth_pov);

    assert!(
        (p_relieved - p_full).abs() < 1e-9,
        "without save pressure the relief must not apply (relieved={p_relieved}, full={p_full})"
    );
}
