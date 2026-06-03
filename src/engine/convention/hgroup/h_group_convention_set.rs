use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;
use crate::engine::convention::hgroup::h_group_core::good_touch_receiver_masks;
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
use crate::game::variant::Variant;

/// An H-Group convention level.
///
/// Each level is a cumulative superset of the levels below it: playing at a
/// given level means using every tech from that level and all lower ones.
///
/// For now each level re-declares the full list of techs it enables, one by
/// one, rather than importing the lower level's list. This keeps each level's
/// roster explicit and easy to read at the cost of some duplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HGroupLevel {
    /// Level 1 — the foundational conventions.
    One,
}

impl HGroupLevel {
    /// The full, cumulative list of techs enabled at this level (including
    /// those inherited from lower levels).
    #[must_use]
    pub fn techs(self) -> Vec<Box<dyn ConventionTech>> {
        match self {
            HGroupLevel::One => vec![
                Box::new(PlayKnownPlayable),
                Box::new(BlindPlay),
                Box::new(DirectPlayClue),
                Box::new(DelayedPlayClue),
                Box::new(TempoClue),
                Box::new(SimplePrompt),
                Box::new(SimpleFinesse),
                Box::new(ColorCriticalSave),
                Box::new(RankCriticalSave),
                Box::new(FiveSave),
                Box::new(TwoSave),
                Box::new(DiscardKnownTrash),
                Box::new(DiscardChop),
            ],
        }
    }
}

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
                Box::new(ClueBurn),
                Box::new(LowLevelStall),
            ],
        }
    }

    /// Build a convention set enabling every tech up to and including `level`.
    #[must_use]
    pub fn for_level(level: HGroupLevel, _variant: &Variant) -> Self {
        HGroupConventionSet::new(level.techs())
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
        focus: Option<CardDeckIndex>,
        receiver: PlayerIndex,
        table_state: &TableState,
        static_data: &StaticGameData,
    ) -> Vec<(CardDeckIndex, VariantCardsBitField)> {
        good_touch_receiver_masks(clue, touched, focus, receiver, table_state, static_data)
    }
}
