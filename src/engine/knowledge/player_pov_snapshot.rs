use crate::engine::game_state_snapshot::GameStateSnapshot;
use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::game::static_game_data::StaticGameData;

/// An owned, per-player snapshot of game state at a specific turn.
///
/// Stores only [`GameStateSnapshot`] (table state + team knowledge) and a player index.
/// `StaticGameData` is intentionally excluded to keep clones cheap — pass it in when you
/// need to materialise a [`LightweightPlayerPOV`] via [`as_pov`](Self::as_pov).
#[derive(Clone)]
pub struct PlayerPOVSnapshot {
    player_index: usize,
    pub snapshot: GameStateSnapshot,
}

impl PlayerPOVSnapshot {
    pub fn new(player_index: usize, snapshot: GameStateSnapshot) -> Self {
        PlayerPOVSnapshot {
            player_index,
            snapshot,
        }
    }

    /// Reconstruct the player's POV for this snapshot, borrowing `static_data` from the caller.
    pub fn as_pov<'a>(&'a self, static_data: &'a StaticGameData) -> LightweightPlayerPOV<'a> {
        self.snapshot.player_pov(self.player_index, static_data)
    }

    pub fn player_index(&self) -> usize {
        self.player_index
    }
}
