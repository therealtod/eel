use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantMetadata {
    pub id: usize,
    #[serde(rename = "newID")]
    pub new_id: String,
    pub name: String,
    pub suits: Vec<String>,
    #[serde(default)]
    pub clue_colors: Vec<String>,
    #[serde(default)]
    pub special_rank: usize,
    #[serde(default)]
    pub special_rank_all_clue_colors: bool,
    #[serde(default)]
    pub special_rank_all_clue_ranks: bool,
    #[serde(default)]
    pub special_rank_no_clue_colors: bool,
    #[serde(default)]
    pub special_rank_no_clue_ranks: bool,
    #[serde(default)]
    pub special_rank_deceptive: bool,
    #[serde(default)]
    pub critical_rank: Option<usize>,
    #[serde(default)]
    pub clue_starved: bool,
    #[serde(default)]
    pub color_clues_touch_nothing: bool,
    #[serde(default)]
    pub rank_clues_touch_nothing: bool,
    #[serde(default)]
    pub alternating_clues: bool,
    #[serde(default)]
    pub cow_and_pig: bool,
    #[serde(default)]
    pub duck: bool,
    #[serde(default)]
    pub odds_and_evens: bool,
    #[serde(default)]
    pub synesthesia: bool,
    #[serde(default)]
    pub up_or_down: bool,
    #[serde(default)]
    pub throw_it_in_a_hole: bool,
    #[serde(default)]
    pub funnels: bool,
    #[serde(default)]
    pub chimneys: bool,
    #[serde(default)]
    pub sudoku: bool,
    #[serde(default = "default_stack_size")]
    pub stack_size: usize,
    #[serde(default = "default_clue_ranks")]
    pub clue_ranks: Vec<usize>,
}

fn default_stack_size() -> usize {
    5
}

fn default_clue_ranks() -> Vec<usize> {
    vec![1, 2, 3, 4, 5]
}
