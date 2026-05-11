use crate::game::state::PlayerIndex;
use crate::game::variant::Variant;

/// Data that stays invariant through the whole game
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticGameData {
    /// Number of players participating in the game
    pub number_of_players: u8,
    /// The [Variant] selected for the game
    pub variant: Variant,
}

impl StaticGameData {
    /// Returns true if `player_a_index` takes their turn before `player_b_index` in the circular
    /// order starting after the active player.
    #[must_use]
    pub fn plays_before(
        &self,
        player_a_index: PlayerIndex,
        player_b_index: PlayerIndex,
        active_player_index: PlayerIndex,
    ) -> bool {
        let n = self.number_of_players as usize;
        let dist = |p: usize| (p + n - active_player_index) % n;
        dist(player_a_index) < dist(player_b_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::variant::test_variants::NO_VARIANT;

    fn make_sgd(num_players: u8) -> StaticGameData {
        StaticGameData {
            number_of_players: num_players,
            variant: NO_VARIANT,
        }
    }

    #[test]
    fn plays_before_basic_3_players() {
        let sgd = make_sgd(3);
        // Active=0: order is 0, 1, 2
        assert!(sgd.plays_before(1, 2, 0));
        assert!(!sgd.plays_before(2, 1, 0));
    }

    #[test]
    fn plays_before_wraps_around() {
        let sgd = make_sgd(3);
        // Active=2: order is 2, 0, 1
        assert!(sgd.plays_before(0, 1, 2));
        assert!(!sgd.plays_before(1, 0, 2));
        // 0 plays before 2? dist(0)=1, dist(2)=0 -> false
        assert!(!sgd.plays_before(0, 2, 2));
        // 2 plays before 0? dist(2)=0, dist(0)=1 -> true
        assert!(sgd.plays_before(2, 0, 2));
    }

    #[test]
    fn plays_before_with_5_players() {
        let sgd = make_sgd(5);
        // Active=4: order is 4, 0, 1, 2, 3
        assert!(sgd.plays_before(0, 3, 4));
        assert!(sgd.plays_before(1, 2, 4));
        assert!(!sgd.plays_before(3, 0, 4));
    }

    #[test]
    fn plays_before_same_player_is_false() {
        let sgd = make_sgd(3);
        assert!(!sgd.plays_before(1, 1, 0));
        assert!(!sgd.plays_before(0, 0, 2));
    }

    #[test]
    fn plays_before_active_player_plays_before_others() {
        let sgd = make_sgd(4);
        // Active=1: order is 1, 2, 3, 0
        assert!(sgd.plays_before(1, 2, 1));
        assert!(sgd.plays_before(1, 3, 1));
        assert!(sgd.plays_before(1, 0, 1));
    }
}
