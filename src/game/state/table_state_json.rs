use crate::game::MAX_HAND_SIZE;
use crate::game::MAX_PLAYERS_IN_GAME;
use crate::game::action::game_action::GameAction;
use crate::game::card::CardDeckIndex;
use crate::game::card::copies_counting_card_collection::CopiesCountingCardCollection;
use crate::game::clue::Clue;
use crate::game::clue_token_bank::ClueTokenBank;
use crate::game::clue_type::ClueType;
use crate::game::deck::Deck;
use crate::game::hand::Hand;
use crate::game::playing_stacks::PlayingStacks;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
use crate::game::variant::Variant;
use serde::Deserialize;
use smallvec::SmallVec;

/// One card slot in a player's hand.
///
/// Two formats are accepted:
///
/// - A plain string `"r3"` — the card identity with no clue history (equivalent to a full
///   entry with empty `positive` and `negative`).
/// - A full object `{"id": "r3", "positive": ["3"], "negative": ["red", "1"], "inferred": "r3"}`.
///
/// `positive` lists clue values that touched this slot (rank digits `"1"`–`"5"` or colour
/// names `"red"`, `"yellow"`, `"green"`, `"blue"`, `"purple"`). Each positive clue narrows
/// the deck empathy to the clue's identity mask, and the slot is marked as clued in
/// `clue_touched_cards`. `negative` lists clue values that were given but did NOT touch this
/// slot, excluding those identities from the empathy mask. This replaces the old parallel
/// `empathy` and `clued_cards` arrays, which fragmented the same (player, slot) data across
/// multiple top-level fields.
///
/// `inferred` stores convention-based knowledge (e.g. from a finesse or save signal) as a
/// concatenated card string (e.g. `"r3b3"`), using the same format as `parse_empathy_mask`.
/// This replaces the old `inferred_identities` parallel array.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SlotJson {
    Simple(String),
    Full(SlotJsonFull),
}

#[derive(Deserialize, Clone, Debug)]
pub struct SlotJsonFull {
    pub id: String,
    #[serde(default)]
    pub positive: Vec<String>,
    #[serde(default)]
    pub negative: Vec<String>,
    pub inferred: Option<String>,
}

impl SlotJson {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            SlotJson::Simple(s) => s,
            SlotJson::Full(f) => &f.id,
        }
    }

    #[must_use]
    pub fn positive(&self) -> &[String] {
        match self {
            SlotJson::Simple(_) => &[],
            SlotJson::Full(f) => &f.positive,
        }
    }

    #[must_use]
    pub fn negative(&self) -> &[String] {
        match self {
            SlotJson::Simple(_) => &[],
            SlotJson::Full(f) => &f.negative,
        }
    }

    #[must_use]
    pub fn inferred(&self) -> Option<&str> {
        match self {
            SlotJson::Simple(_) => None,
            SlotJson::Full(f) => f.inferred.as_deref(),
        }
    }
}

/// An action recorded in `prior_actions`.
///
/// The loader replays these after the base scenario state to build `Vec<GameStateSnapshot>`
/// history and the corresponding `Vec<GameAction>`, so tests do not need to hardcode actions
/// or manually fabricate history. For `Clue` actions, touched cards are computed
/// automatically from the receiver's hand and the clue's empathy mask (`"x"` cards are
/// skipped).
///
/// `slot` in `Play` and `Discard` is 1-indexed (slot 1 = newest card).
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PriorActionJson {
    Clue {
        giver: usize,
        receiver: usize,
        /// Rank digit (`"1"`–`"5"`) or colour name (`"red"`, `"yellow"`, `"green"`,
        /// `"blue"`, `"purple"`).
        clue: String,
    },
    Play {
        player: usize,
        slot: usize,
    },
    Discard {
        player: usize,
        slot: usize,
    },
}

/// JSON representation of a scenario.
///
/// Example:
/// ```json
/// {
///   "suits": ["red", "yellow", "green", "blue", "purple"],
///   "playing_stacks": [["r1"], [], ["g1", "g2"], [], []],
///   "discard_pile": ["p4", "r2"],
///   "clue_tokens": 5,
///   "strikes": 0,
///   "active_player": 0,
///   "hands": [
///     ["r3", "b2", "g4", "y1", "p2"],
///     [{"id": "b3", "positive": ["3"], "negative": ["red"]}, "r5", "y2", "g3", "p1"],
///     ...
///   ],
///   "prior_actions": [
///     {"type": "clue", "giver": 0, "receiver": 1, "clue": "3"}
///   ]
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
    pub active_player: usize,
    /// Each element is one player's hand, listed slot1-first (newest card first, oldest last).
    /// Each slot may be a plain card string (`"r3"`) or a full `SlotJson` object with optional
    /// positive/negative clue-touch information and an inferred identity.
    pub hands: Vec<Vec<SlotJson>>,
    /// Actions that happen after the base scenario state, replayed in order by the loader to
    /// produce `Vec<GameStateSnapshot>` history and `Vec<GameAction>` alongside the base state.
    #[serde(default)]
    pub prior_actions: Vec<PriorActionJson>,
}

/// Parse a card string like `"r1"`, `"b3"`, `"p4"` into a `VariantCardId` for no-variant.
///
/// No-variant card IDs:
///   R1=0, R2=1, R3=2, R4=3, R5=4
///   Y1=5, Y2=6, Y3=7, Y4=8, Y5=9
///   G1=10, G2=11, G3=12, G4=13, G5=14
///   B1=15, B2=16, B3=17, B4=18, B5=19
///   P1=20, P2=21, P3=22, P4=23, P5=24
#[must_use]
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
#[must_use]
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

/// Parse a clue value string into `(ClueType, value)`.
///
/// Rank clues: `"1"`–`"5"` → `(Rank, rank)`.
/// Colour clues: `"red"` / `"r"`, `"yellow"` / `"y"`, `"green"` / `"g"`,
/// `"blue"` / `"b"`, `"purple"` / `"pink"` / `"p"` → `(Color, suit_index)`.
#[must_use]
pub fn parse_clue_string(s: &str) -> (ClueType, u8) {
    match s.to_lowercase().as_str() {
        "red" | "r" => (ClueType::Color, 0),
        "yellow" | "y" => (ClueType::Color, 1),
        "green" | "g" => (ClueType::Color, 2),
        "blue" | "b" => (ClueType::Color, 3),
        "purple" | "pink" | "p" => (ClueType::Color, 4),
        other => {
            let rank: u8 = other
                .parse()
                .unwrap_or_else(|_| panic!("invalid clue value '{s}'"));
            (ClueType::Rank, rank)
        }
    }
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

/// Returns hands array. JSON is slot1-first (newest→oldest); deck indices are assigned
/// oldest-first so the oldest card (rightmost in JSON) gets the lowest index.
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
        for (slot_pos, slot) in player_hand.iter().enumerate() {
            let deck_index = base_index + hand_size - 1 - slot_pos as u8;
            let id = slot.id();
            if id != "x" {
                deck.reveal_card(deck_index, parse_card(id));
            }
        }
        base_index += hand_size;
    }
    deck
}

/// Applies per-slot positive and negative clue touches, updating deck empathy and
/// `clue_touched_cards`. Replaces the old separate `empathy` and `clued_cards` fields.
fn apply_slot_clues(
    scenario: &ScenarioJson,
    deck: &mut Deck,
    clue_touched_cards: &mut u64,
    variant: &Variant,
) {
    let mut base_index: u8 = 0;
    for player_hand in &scenario.hands {
        let hand_size = player_hand.len() as u8;
        for (slot_pos, slot) in player_hand.iter().enumerate() {
            let deck_index = base_index + hand_size - 1 - slot_pos as u8;

            for clue_str in slot.positive() {
                let (ct, cv) = parse_clue_string(clue_str);
                let mask = variant.empathy_by_clue(ct, cv as usize).as_bits();
                deck.update_positive_empathy(deck_index, mask);
                *clue_touched_cards |= 1u64 << deck_index;
            }

            for clue_str in slot.negative() {
                let (ct, cv) = parse_clue_string(clue_str);
                let mask = variant.empathy_by_clue(ct, cv as usize).as_bits();
                deck.update_negative_empathy(deck_index, mask);
            }
        }
        base_index += hand_size;
    }
}

/// Convert `scenario.prior_actions` into `GameAction`s.
///
/// Touched cards for `Clue` entries are computed from the receiver's hand identities and the
/// clue's empathy mask. `"x"` (unknown) cards are skipped. The `turn` field on each action
/// matches its index in `prior_actions`, so `actions[i]` has `turn = i` and the loader can
/// record `history[i]` as the snapshot taken before that action.
#[must_use]
pub fn build_game_actions(scenario: &ScenarioJson, variant: &Variant) -> Vec<GameAction> {
    let player_bases: Vec<u8> = {
        let mut base = 0u8;
        scenario
            .hands
            .iter()
            .map(|h| {
                let b = base;
                base += h.len() as u8;
                b
            })
            .collect()
    };

    scenario
        .prior_actions
        .iter()
        .enumerate()
        .map(|(turn, action)| match action {
            PriorActionJson::Clue {
                receiver,
                clue: clue_str,
                ..
            } => {
                let (clue_type, clue_value) = parse_clue_string(clue_str);
                let clue = Clue {
                    clue_type,
                    clue_value,
                };
                let clue_mask = variant.empathy_for_clue(&clue).as_bits();

                let receiver_hand = &scenario.hands[*receiver];
                let base = player_bases[*receiver];
                let hand_size = receiver_hand.len() as u8;

                let touched: SmallVec<[CardDeckIndex; MAX_HAND_SIZE]> = receiver_hand
                    .iter()
                    .enumerate()
                    .filter_map(|(slot_pos, slot)| {
                        let id = slot.id();
                        if id == "x" {
                            return None;
                        }
                        let card_id = parse_card(id);
                        if clue_mask & (1 << card_id) != 0 {
                            Some(base + hand_size - 1 - slot_pos as u8)
                        } else {
                            None
                        }
                    })
                    .collect();

                GameAction::Clue {
                    player_index: *receiver,
                    touched_card_deck_indexes: touched,
                    clue,
                    turn,
                }
            }
            PriorActionJson::Play { player, slot } => {
                let base = player_bases[*player];
                let hand_size = scenario.hands[*player].len() as u8;
                GameAction::Play {
                    player_index: *player,
                    card_deck_index: base + hand_size - *slot as u8,
                    turn,
                }
            }
            PriorActionJson::Discard { player, slot } => {
                let base = player_bases[*player];
                let hand_size = scenario.hands[*player].len() as u8;
                GameAction::Discard {
                    player_index: *player,
                    card_deck_index: base + hand_size - *slot as u8,
                    turn,
                }
            }
        })
        .collect()
}

/// Build a `(TableState, StaticGameData)` pair from a `ScenarioJson` and a `Variant`.
#[must_use]
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

    let mut clue_touched_cards: u64 = 0;
    apply_slot_clues(
        scenario,
        &mut deck,
        &mut clue_touched_cards,
        &static_game_data.variant,
    );

    let mut table_state = TableState::from_parts(
        ClueTokenBank::new_from_whole_tokens(scenario.clue_tokens),
        deck,
        hands,
        scenario.active_player,
        0, // turn_counter
        playing_stacks,
        scenario.strikes,
        discard_pile,
    );

    table_state.all_hand_bits = table_state.hands[..static_game_data.number_of_players as usize]
        .iter()
        .flat_map(|h| h.cards())
        .fold(0u64, |acc, &idx| acc | (1u64 << idx));

    table_state.clue_touched_cards = clue_touched_cards;

    (table_state, static_game_data)
}

/// Load a `ScenarioJson` from a JSON string.
#[must_use]
pub fn parse_scenario(json: &str) -> ScenarioJson {
    serde_json::from_str(json).expect("failed to parse scenario JSON")
}
