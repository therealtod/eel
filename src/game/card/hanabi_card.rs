use crate::game::rank::Rank;
use crate::game::suit::Suit;
use std::fmt;

#[derive(PartialEq, Debug)]
pub struct HanabiCard {
    pub suit: Suit,
    pub rank: Rank,
}

impl fmt::Display for HanabiCard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.suit, self.rank)
    }
}
