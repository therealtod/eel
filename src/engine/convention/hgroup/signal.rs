use crate::engine::knowledge::knowledge_update::KnowledgeUpdate;
use crate::game::SlotIndex;
use crate::game::card::CardDeckIndex;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Signal {
    Discard {
        slot_index: SlotIndex,
        turn: usize,
    },
    Play {
        card_deck_index: CardDeckIndex,
        knowledge_updates: Vec<KnowledgeUpdate>,
    },
    Save {
        slot_index: SlotIndex,
        turn: usize,
    },
}
