use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::good_touch_baseline_mask;
use crate::game::card::{CardDeckIndex, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// The H-Group convention framework.
pub struct HGroupConventionSet {
    techs: Vec<Box<dyn ConventionTech>>,
}

impl HGroupConventionSet {
    #[must_use]
    pub fn new(mut techs: Vec<Box<dyn ConventionTech>>) -> Self {
        techs.sort_by_key(|t| t.interpretation_priority());
        HGroupConventionSet { techs }
    }
}

impl ConventionSet for HGroupConventionSet {
    fn techs(&self) -> &[Box<dyn ConventionTech>] {
        &self.techs
    }

    fn clue_receiver_baseline(
        &self,
        clue: &Clue,
        touched: &[CardDeckIndex],
        receiver: PlayerIndex,
        table_state: &TableState,
        static_data: &StaticGameData,
    ) -> Vec<(CardDeckIndex, VariantCardsBitField)> {
        let Some(mask) = good_touch_baseline_mask(clue, receiver, table_state, static_data) else {
            return Vec::new();
        };
        touched.iter().map(|&idx| (idx, mask)).collect()
    }
}
