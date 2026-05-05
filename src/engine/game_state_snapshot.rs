use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;

/// A snapshot of the game state at a specific turn: the observable board state plus all
/// per-player knowledge at that moment.
///
/// `StaticGameData` is excluded because it never changes across a game; callers supply it
/// when reconstructing a [`LightweightPlayerPOV`].
#[derive(Clone)]
pub struct GameStateSnapshot {
    pub table_state: TableState,
    pub team_knowledge: TeamKnowledge,
}

impl GameStateSnapshot {
    pub fn new(table_state: TableState, team_knowledge: TeamKnowledge) -> Self {
        GameStateSnapshot {
            table_state,
            team_knowledge,
        }
    }

    pub fn player_pov<'a>(
        &'a self,
        player_index: usize,
        static_data: &'a StaticGameData,
    ) -> LightweightPlayerPOV<'a> {
        LightweightPlayerPOV::new(
            player_index,
            self.team_knowledge.player(player_index),
            &self.team_knowledge,
            &self.table_state,
            static_data,
        )
    }
}
