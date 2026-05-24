use crate::game::clue_type::ClueType;

/// Represents the action of giving a clue in a game
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Clue {
    pub clue_type: ClueType,
    pub clue_value: u8,
}

impl Clue {
    pub const RANK_1_CLUE: Clue = Clue {
        clue_type: ClueType::Rank,
        clue_value: 1,
    };

    pub const RANK_2_CLUE: Clue = Clue {
        clue_type: ClueType::Rank,
        clue_value: 2,
    };

    pub const RANK_5_CLUE: Clue = Clue {
        clue_type: ClueType::Rank,
        clue_value: 5,
    };
}
