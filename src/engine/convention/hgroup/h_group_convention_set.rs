use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::good_touch_baseline_mask;
use crate::engine::convention::hgroup::tech::blind_play::BlindPlay;
use crate::engine::convention::hgroup::tech::clue_burn::ClueBurn;
use crate::engine::convention::hgroup::tech::critical_save::{ColorCriticalSave, RankCriticalSave};
use crate::engine::convention::hgroup::tech::delayed_play_clue::DelayedPlayClue;
use crate::engine::convention::hgroup::tech::direct_play_clue::DirectPlayClue;
use crate::engine::convention::hgroup::tech::discard_chop::DiscardChop;
use crate::engine::convention::hgroup::tech::discard_known_trash::DiscardKnownTrash;
use crate::engine::convention::hgroup::tech::five_save::FiveSave;
use crate::engine::convention::hgroup::tech::five_stall::FiveStall;
use crate::engine::convention::hgroup::tech::low_level_stall::LowLevelStall;
use crate::engine::convention::hgroup::tech::play_known_playable::PlayKnownPlayable;
use crate::engine::convention::hgroup::tech::simple_finesse::SimpleFinesse;
use crate::engine::convention::hgroup::tech::simple_prompt::SimplePrompt;
use crate::engine::convention::hgroup::tech::tempo_clue::TempoClue;
use crate::engine::convention::hgroup::tech::two_save::TwoSave;
use crate::game::card::{CardDeckIndex, VariantCardsBitField};
use crate::game::clue::Clue;
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// The H-Group convention framework.
pub struct HGroupConventionSet {
    techs: Vec<Box<dyn ConventionTech>>,
    /// Fallback techs consulted when no primary tech has candidate actions.
    /// Stored in fallback-chain order (first match wins) — NOT sorted by priority.
    fallback_techs: Vec<Box<dyn ConventionTech>>,
}

impl HGroupConventionSet {
    #[must_use]
    pub fn new(mut techs: Vec<Box<dyn ConventionTech>>) -> Self {
        techs.sort_by_key(|t| t.interpretation_priority());
        HGroupConventionSet {
            techs,
            fallback_techs: vec![
                Box::new(FiveStall),
                Box::new(TempoClue),
                Box::new(ClueBurn),
                Box::new(LowLevelStall),
            ],
        }
    }
}

impl Default for HGroupConventionSet {
    fn default() -> Self {
        HGroupConventionSet::new(vec![
            Box::new(PlayKnownPlayable),
            Box::new(BlindPlay),
            Box::new(DirectPlayClue),
            Box::new(DelayedPlayClue),
            Box::new(SimplePrompt),
            Box::new(SimpleFinesse),
            Box::new(ColorCriticalSave),
            Box::new(RankCriticalSave),
            Box::new(FiveSave),
            Box::new(TwoSave),
            Box::new(DiscardKnownTrash),
            Box::new(DiscardChop),
        ])
    }
}

impl ConventionSet for HGroupConventionSet {
    fn techs(&self) -> &[Box<dyn ConventionTech>] {
        &self.techs
    }

    fn fallback_techs(&self) -> &[Box<dyn ConventionTech>] {
        &self.fallback_techs
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
