use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuitMetadata {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub abbreviation: String,
    #[serde(default)]
    pub fill: Option<String>,
    #[serde(default)]
    pub fill_colors: Vec<String>,
    #[serde(default)]
    pub clue_colors: Vec<String>,
    #[serde(default)]
    pub create_variants: bool,
    pub pip: String,
    #[serde(default)]
    pub prism: bool,
    #[serde(default)]
    pub one_of_each: bool,
    #[serde(default)]
    pub all_clue_colors: bool,
    #[serde(default)]
    pub all_clue_ranks: bool,
    #[serde(default)]
    pub no_clue_colors: bool,
    #[serde(default)]
    pub no_clue_ranks: bool,
}
