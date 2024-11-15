use crate::game::MAX_PLAYERS_IN_GAME;
use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
use serde::Deserialize;

/// JSON representation of a scenario, mirroring the Kotlin YAML format.
///
/// Example:
/// ```json
/// {
///   "suits": ["red", "yellow", "green", "blue", "purple"],
///   "playing_stacks": [["r1"], [], ["g1", "g2"], [], []],
///   "discard_pile": ["p4", "r2"],
///   "clue_tokens": 5,
///   "strikes": 0,
///   "player_on_turn": 0,
///   "hands": [["r3", "b2", "g4", "y1", "p2"], ["b3", "r5", "y2", "g3", "p1"], ...]
/// }
/// ```
#[derive(Deserialize)]
pub struct ScenarioJson {
    #[serde(default)]
    pub scenario_description: String,
    pub suits: Vec<String>,
    pub playing_stacks: Vec<Vec<String>>,
    pub discard_pile: Vec<String>,
    pub clue_tokens: u8,
    pub strikes: u8,
    pub player_on_turn: usize,
    /// Each element is one player's hand, listed slot1-first (newest card first, oldest last).
    /// Unknown cards can be represented as `"x"` and will be assigned a placeholder deck index.
    pub hands: Vec<Vec<String>>,
    /// Cards that have been touched by a clue, as `[player_index, slot]` pairs (slot is 1-indexed).
    /// Optional; defaults to empty.
    #[serde(default)]
    pub clued_cards: Vec<[usize; 2]>,
    /// Per-player, per-slot empathy: what each player believes about their own hand.
    /// Outer index = player, inner index = slot (slot1-first, same order as `hands`).
    /// Each entry is a string of concatenated card identities the player considers possible,
    /// e.g. `"r1"` (known exactly), `"b3b4"` (either b3 or b4), or `null`/omitted (unknown).
    /// The entire field is optional; missing players or slots default to fully unknown.
    /// Use `"x"` for a slot with no information (same convention as `hands`).
    #[serde(default)]
    pub empathy: Vec<Vec<String>>,
}

/// Parse a card string like `"r1"`, `"b3"`, `"p4"` into a `VariantCardId` for no-variant.
///
/// No-variant card IDs:
///   R1=0, R2=1, R3=2, R4=3, R5=4
///   Y1=5, Y2=6, Y3=7, Y4=8, Y5=9
///   G1=10, G2=11, G3=12, G4=13, G5=14
///   B1=15, B2=16, B3=17, B4=18, B5=19
///   P1=20, P2=21, P3=22, P4=23, P5=24
pub fn parse_card(s: &str) -> usize {
    let s = s.to_lowercase();
    let suit_offset = match s.chars().next().expect("empty card string") {
        'r' => 0,
        'y' => 5,
        'g' => 10,
        'b' => 15,
        'p' => 20,
        c => panic!("unknown suit '{c}' in card '{s}'"),
    };
    let rank: usize = s[1..].parse().expect("invalid rank in card string");
    suit_offset + rank - 1
}

/// Parse a concatenated empathy string like `"b3b4"` into a bitmask of possible card identities.
///
/// The string is split greedily: each card is one letter (suit) followed by one digit (rank).
pub fn parse_empathy_mask(s: &str) -> u64 {
    let s = s.to_lowercase();
    let bytes = s.as_bytes();
    let mut mask: u64 = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        let card_str = std::str::from_utf8(&bytes[i..i + 2]).unwrap();
        mask |= 1 << parse_card(card_str);
        i += 2;
    }
    mask
}

fn build_playing_stacks(scenario: &ScenarioJson, variant: &Variant) -> PlayingStacks {
    let played_cards: Vec<usize> = scenario
        .playing_stacks
        .iter()
        .flat_map(|stack| stack.iter().map(|c| parse_card(c)))
        .collect();
    PlayingStacks::new(played_cards, variant)
}

fn build_discard_pile(scenario: &ScenarioJson) -> CopiesCountingCardCollection {
    let mut discard_pile = CopiesCountingCardCollection::empty();
    for card_str in &scenario.discard_pile {
        discard_pile.add_card_with_id(parse_card(card_str));
    }
    discard_pile
}

/// Returns hands array and the base deck index for each player (oldest-first assignment).
/// JSON is slot1-first (newest→oldest); deck indices are assigned oldest-first so the
/// oldest card (rightmost in JSON) gets the lowest index.
fn build_hands(scenario: &ScenarioJson) -> [Hand; MAX_PLAYERS_IN_GAME] {
    let mut hands = core::array::from_fn(|_| Hand::empty());
    let mut next_deck_index: u8 = 0;
    for (player_idx, player_hand) in scenario.hands.iter().enumerate().take(MAX_PLAYERS_IN_GAME) {
        let hand_size = player_hand.len() as u8;
        let oldest_first: Vec<u8> = (next_deck_index..next_deck_index + hand_size).collect();
        hands[player_idx] = Hand::new(&oldest_first);
        next_deck_index += hand_size;
    }
    hands
}

/// Builds the deck, decrements it by dealt cards, and reveals all known cards.
fn build_deck(scenario: &ScenarioJson, variant: &Variant) -> Deck {
    let mut deck = Deck::new(variant);
    let dealt: u8 = scenario.hands.iter().map(|h| h.len() as u8).sum();
    deck.decrement_size(dealt);

    let mut base_index: u8 = 0;
    for player_hand in &scenario.hands {
        let hand_size = player_hand.len() as u8;
        for (slot_pos, card_str) in player_hand.iter().enumerate() {
            // slot_pos 0 = slot 1 (newest) → deck index = base + hand_size - 1
            let deck_index = base_index + hand_size - 1 - slot_pos as u8;
            if card_str != "x" {
                deck.reveal_card(deck_index, parse_card(card_str));
            }
        }
        base_index += hand_size;
    }
    deck
}

/// Marks clued cards as touched. `clued_cards` entries are `[player_index, slot]`
/// where slot is 1-indexed (slot 1 = newest card).
fn apply_clued_cards(scenario: &ScenarioJson, table_state: &mut TableState) {
    for &[player_idx, slot] in &scenario.clued_cards {
        if player_idx >= scenario.hands.len() || slot == 0 {
            continue;
        }
        let hand = &scenario.hands[player_idx];
        let json_idx = slot - 1; // slot 1 → index 0 (newest)
        if json_idx >= hand.len() {
            continue;
        }
        let base: u8 = scenario.hands[..player_idx]
            .iter()
            .map(|h| h.len() as u8)
            .sum();
        let deck_idx = base + hand.len() as u8 - 1 - json_idx as u8;
        table_state.clue_touched_cards |= 1u64 << deck_idx;
    }
}

/// Applies per-slot empathy overrides from the scenario onto the deck.
/// Outer index = player, inner index = slot (slot1-first, same order as `hands`).
fn apply_empathy(scenario: &ScenarioJson, deck: &mut Deck) {
    let mut base_index: u8 = 0;
    for (player_idx, player_hand) in scenario.hands.iter().enumerate() {
        let hand_size = player_hand.len() as u8;
        if let Some(player_empathy) = scenario.empathy.get(player_idx) {
            for (slot_pos, empathy_str) in player_empathy.iter().enumerate() {
                if slot_pos >= player_hand.len() || empathy_str == "x" {
                    continue;
                }
                let deck_index = base_index + hand_size - 1 - slot_pos as u8;
                let mask = parse_empathy_mask(empathy_str);
                if mask != 0 {
                    deck.update_positive_empathy(deck_index, mask);
                }
            }
        }
        base_index += hand_size;
    }
}

/// Build a `(TableState, StaticGameData)` pair from a `ScenarioJson` and a `Variant`.
pub fn build_from_scenario(
    scenario: &ScenarioJson,
    variant: Variant,
) -> (TableState, StaticGameData) {
    let static_game_data = StaticGameData {
        number_of_players: scenario.hands.len() as u8,
        variant,
    };

    let playing_stacks = build_playing_stacks(scenario, &static_game_data.variant);
    let discard_pile = build_discard_pile(scenario);
    let hands = build_hands(scenario);
    let mut deck = build_deck(scenario, &static_game_data.variant);
    apply_empathy(scenario, &mut deck);

    let mut table_state = TableState::from_parts(
        ClueTokenBank::new_from_whole_tokens(scenario.clue_tokens),
        deck,
        hands,
        scenario.player_on_turn,
        playing_stacks,
        scenario.strikes,
        discard_pile,
    );

    table_state.all_hand_bits = table_state.hands[..static_game_data.number_of_players as usize]
        .iter()
        .flat_map(|h| h.cards())
        .fold(0u64, |acc, &idx| acc | (1u64 << idx));

    apply_clued_cards(scenario, &mut table_state);

    (table_state, static_game_data)
}

/// Load a `ScenarioJson` from a JSON string.
pub fn parse_scenario(json: &str) -> ScenarioJson {
    serde_json::from_str(json).expect("failed to parse scenario JSON")
}
