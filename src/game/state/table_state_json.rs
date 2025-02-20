use serde::Deserialize;
use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
use crate::game::MAX_PLAYERS_IN_GAME;

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
    pub suits: Vec<String>,
    pub playing_stacks: Vec<Vec<String>>,
    pub discard_pile: Vec<String>,
    pub clue_tokens: u8,
    pub strikes: u8,
    pub player_on_turn: usize,
    /// Each element is one player's hand (oldest card first, newest last).
    /// Unknown cards can be represented as `"x"` and will be assigned a placeholder deck index.
    pub hands: Vec<Vec<String>>,
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

/// Build a `(TableState, StaticGameData)` pair from a `ScenarioJson` and a `Variant`.
pub fn build_from_scenario(scenario: &ScenarioJson, variant: Variant) -> (TableState, StaticGameData) {
    let number_of_players = scenario.hands.len() as u8;
    let static_game_data = StaticGameData { number_of_players, variant };

    // Build playing stacks
    let played_cards: Vec<usize> = scenario
        .playing_stacks
        .iter()
        .flat_map(|stack| stack.iter().map(|c| parse_card(c)))
        .collect();
    let playing_stacks = PlayingStacks::new(played_cards, &static_game_data.variant);

    // Build discard pile
    let mut discard_pile = CopiesCountingCardCollection::empty();
    for card_str in &scenario.discard_pile {
        discard_pile.add_card_with_id(parse_card(card_str));
    }

    // Assign deck indexes sequentially; each card slot gets a unique index.
    let mut next_deck_index: u8 = 0;
    let mut hands = [
        Hand::empty(),
        Hand::empty(),
        Hand::empty(),
        Hand::empty(),
        Hand::empty(),
        Hand::empty(),
    ];
    for (player_idx, player_hand) in scenario.hands.iter().enumerate() {
        if player_idx >= MAX_PLAYERS_IN_GAME {
            break;
        }
        let card_indexes: Vec<u8> = player_hand
            .iter()
            .map(|_| {
                let idx = next_deck_index;
                next_deck_index += 1;
                idx
            })
            .collect();
        hands[player_idx] = Hand::new(card_indexes);
    }

    // Build deck and reveal known cards
    let mut deck = Deck::new(&static_game_data.variant);
    let dealt: u8 = scenario.hands.iter().map(|h| h.len() as u8).sum();
    deck.decrement_size(dealt);

    let mut deck_index: u8 = 0;
    for player_hand in &scenario.hands {
        for card_str in player_hand {
            if card_str != "x" {
                deck.reveal_card(deck_index, parse_card(card_str));
            }
            deck_index += 1;
        }
    }

    let clue_token_bank = ClueTokenBank::new(scenario.clue_tokens * 2);

    let table_state = TableState::from_parts(
        clue_token_bank,
        deck,
        hands,
        scenario.player_on_turn,
        playing_stacks,
        scenario.strikes,
        discard_pile,
    );

    (table_state, static_game_data)
}

/// Load a `ScenarioJson` from a JSON string.
pub fn parse_scenario(json: &str) -> ScenarioJson {
    serde_json::from_str(json).expect("failed to parse scenario JSON")
}
