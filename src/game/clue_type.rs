#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClueType {
    Color,
    Rank,
}

impl From<ClueType> for usize {
    fn from(value: ClueType) -> Self {
        match value {
            ClueType::Color => 0,
            ClueType::Rank => 1,
        }
    }
}