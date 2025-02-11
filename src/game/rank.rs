use std::fmt;
#[derive(PartialEq, Debug)]
pub enum Rank {
    Start,
    One,
    Two,
    Three,
    Four,
    Five,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rank::Start => write!(f, "START"),
            Rank::One => write!(f, "1"),
            Rank::Two => write!(f, "2"),
            Rank::Three => write!(f, "3"),
            Rank::Four => write!(f, "4"),
            Rank::Five => write!(f, "5"),
        }
    }
}
