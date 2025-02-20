use crate::game::SlotIndex;


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Signal {
    Discard {slot_index: SlotIndex, turn: usize},
    Play{slot_index: SlotIndex, turn: usize},
    Save{slot_index: SlotIndex, turn: usize},
}