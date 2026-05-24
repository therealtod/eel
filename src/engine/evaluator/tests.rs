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
    let score_correct =
        DefaultEvaluator::misinformation_score(&static_data, &tk, &state, &truth);
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
    let score =
        DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
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
    let score =
        DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
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
    let score =
        DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
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
    let score =
        DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
    assert_eq!(score, 0.0, "no critical identities → zero exposure");
}

#[test]
fn critical_exposure_zero_when_critical_is_touched() {
    let (mut state, static_data, tk) = make_chop_test_state([4, 7, 12, 17, 22]);
    state.clue_touched_cards |= 1u64 << 0;
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(&state, &static_data);
    let score =
        DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
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
    let score =
        DefaultEvaluator::critical_exposure_score(&state, &static_data, &tk, &truth_pov);
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
    let pre = KnowledgeAwareGameState::from_parts(
        static_data.clone(),
        pre_state.clone(),
        tk.clone(),
        11,
    );

    let post_unsaved = KnowledgeAwareGameState::from_parts(
        static_data.clone(),
        pre_state.clone(),
        tk.clone(),
        11,
    );

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
    let delta =
        evaluator.critical_exposure_delta_bonus(&action, 0, &pre, &post_saved, &truth_pov);
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
    pk1.signals[11].push(Signal::Play { card_deck_index: 11, committed_identity: 0 });
    pk1.signals[12].push(Signal::Play { card_deck_index: 12, committed_identity: 5 });
    *tk.player_mut(1) = pk1;

    let pre = KnowledgeAwareGameState::from_parts(
        static_data.clone(),
        pre_state.clone(),
        tk.clone(),
        13,
    );
    let post_unsaved = pre.clone();
    let mut saved_state = pre_state.clone();
    saved_state.clue_touched_cards |= 1u64 << 10;
    let post_saved =
        KnowledgeAwareGameState::from_parts(static_data.clone(), saved_state, tk, 13);

    let weight = 2.0_f64;
    let evaluator = DefaultEvaluator {
        critical_exposure_delta_weight: weight,
        ..DefaultEvaluator::default()
    };
    let truth = TruthFixture::new(&static_data);
    let truth_pov = truth.pov(pre.table_state(), &static_data);
    let discard = GameAction::Discard { player_index: 0, card_deck_index: 0, turn: 1 };
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
    let post = make_kags_with_stacks(0);
    let action = GameAction::Play {
        player_index: 0,
        card_deck_index: 0,
        turn: 1,
    };
    let cost = evaluator.tempo_cost(&action, &pre, &post, 0, 0);
    assert_eq!(
        cost,
        -evaluator.tempo_cost_weight,
        "misplay (no score advance) must pay tempo cost"
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
    assert_eq!(cost, -evaluator.tempo_cost_weight, "clue must pay tempo cost");
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
