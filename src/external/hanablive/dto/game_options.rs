use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameOptions {
    pub max_players: Option<usize>,
    pub num_players: usize,
    pub starting_player: usize,
    #[serde(rename = "variantID")]
    pub variant_id: usize,
    pub variant_name: String,
    pub table_name: Option<String>,
    pub timed: bool,
    pub time_base: usize,
    pub time_per_turn: usize,
    pub speedrun: bool,
    pub card_cycle: bool,
    pub deck_plays: bool,
    pub empty_clues: bool,
    pub one_extra_card: bool,
    pub one_less_card: bool,
    pub all_or_nothing: bool,
    pub detrimental_characters: bool,
}
