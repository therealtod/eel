use std::fmt;

#[derive(PartialEq, Debug)]
pub struct Suit {}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "some_suit")
    }
}
pub type SuitId = String;
