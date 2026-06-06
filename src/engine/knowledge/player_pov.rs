use crate::engine::knowledge::lightweight_player_pov::LightweightPlayerPOV;
use crate::engine::knowledge::team_knowledge::TeamKnowledge;
use crate::game::action::game_action::GameAction;
use crate::game::card::{
    CardDeckIndex, CardIdentityMask, DeckCardsBitField, VariantCardId, VariantCardsBitField,
};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::static_game_data::StaticGameData;
#[cfg(test)]
use mockall::automock;

/// Read-only view of the knowledge that a specific player has on the state of the game.
///
/// Covers public information (board state) as well as private information (teammate hands,
/// inferred identities, convention signals, etc.).
///
/// This trait is intentionally **read-only**: mutations go through
/// [`PlayerKnowledgeState`](super::player_knowledge::PlayerKnowledgeState) directly.
#[cfg_attr(test, automock)]
pub trait PlayerPOV {
    /// Get the player's own player index
    fn player_index(&self) -> PlayerIndex;

    /// Get the "away value" of a card (how many plays away from being playable).
    fn away_value(&self, card_id: VariantCardId) -> Option<u8>;

    /// Get the known identity of a card from this POV, if fully determined.
    fn card_identity(&self, card_deck_index: CardDeckIndex) -> Option<VariantCardId>;

    fn own_playable_cards(&self) -> DeckCardsBitField;

    /// Canonical "known playable" predicate: the holder of `card_deck_index` knows it
    /// is currently playable, either because they carry a [`Signal::Play`] on it, or
    /// because every identity their empathy still allows is inside `playable_mask`
    /// (an empty mask — a contradiction — is never known-playable).
    ///
    /// This is the single source of truth for "known playable" across the engine:
    /// [`is_playable`](Self::is_playable) (and hence [`is_hand_locked`](Self::is_hand_locked))
    /// delegate here after computing the mask, and the evaluator's
    /// `DefaultEvaluator::card_known_playable` delegates here directly (passing a
    /// precomputed mask to avoid recomputing it per card in hot paths).
    ///
    /// The signal source is `team_knowledge().player(player_index())`, which equals the
    /// POV's own knowledge under the construction invariant (a POV's knowledge is its
    /// entry in `team_knowledge`).
    fn is_playable_with_playable_mask(
        &self,
        card_deck_index: CardDeckIndex,
        playable_mask: VariantCardsBitField,
    ) -> bool {
        let pk = self.team_knowledge().player(self.player_index());
        if pk.has_play_signal(card_deck_index) {
            return true;
        }
        let bits = self.inferred_identities(card_deck_index).as_bits();
        bits != 0 && (bits & playable_mask) == bits
    }

    /// Whether the holder knows `card_deck_index` is currently playable. Computes the
    /// current playable mask and delegates to [`Self::is_playable_with_playable_mask`].
    fn is_playable(&self, card_deck_index: CardDeckIndex) -> bool {
        let playable_mask = self.table_state().playable_cards(self.static_data());
        self.is_playable_with_playable_mask(card_deck_index, playable_mask)
    }

    /// Touched by a clue: the card has at least one positive clue mark on it
    /// (i.e. the bit is set in `table_state.clue_touched_cards`).
    fn is_clue_touched(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Touched by a convention play signal: the card's holder carries a
    /// `Signal::Play` on this deck index.
    fn is_signal_touched(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Touched in any sense — clue-touched OR signal-touched.
    fn is_touched(&self, card_deck_index: CardDeckIndex) -> bool {
        self.is_clue_touched(card_deck_index) || self.is_signal_touched(card_deck_index)
    }

    /// Returns true if this POV's player has a **locked hand**: every card in
    /// their hand is touched (clue- or signal-touched) and none of them is known
    /// to be playable or known trash.
    ///
    /// A player with a locked hand has no safe discard and no known play — every
    /// card is "stuck" awaiting further information.
    ///
    /// An empty hand is not considered locked.
    #[allow(clippy::cast_possible_truncation)]
    fn is_hand_locked(&self) -> bool {
        let mut hand = self.team_knowledge().player(self.player_index()).own_hand;
        if hand == 0 {
            return false;
        }
        while hand != 0 {
            let card_deck_index = hand.trailing_zeros() as CardDeckIndex;
            hand &= hand - 1;
            if !self.is_touched(card_deck_index) {
                return false;
            }
            if self.is_playable(card_deck_index) || self.is_known_trash(card_deck_index) {
                return false;
            }
        }
        true
    }

    /// Returns true if the holder of this card knows its exact identity
    /// (i.e. it is in their `visible_cards`, meaning it was revealed to them via a clue or inference).
    fn is_identity_known_to_holder(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Returns true if the card is the last remaining copy (all other copies have been discarded).
    fn is_critical(&self, card_deck_index: CardDeckIndex) -> bool;

    fn is_critical_card_id(&self, variant_card_id: VariantCardId) -> bool;

    fn active_player_index(&self) -> PlayerIndex;

    fn table_state(&self) -> &TableState;

    fn static_data(&self) -> &StaticGameData;

    fn team_knowledge(&self) -> &TeamKnowledge;

    /// Effective sight set for this POV. Defaults to the holder's stored
    /// `visible_cards`; search-time teammate POVs intersect this with the root
    /// observer's sight to avoid omniscient peeking during rollouts.
    fn visible_cards(&self) -> DeckCardsBitField;

    /// Reconstruct the POV of `player_index` using the knowledge this player has about them.
    ///
    /// The returned view uses `team_knowledge().player(player_index)` as the knowledge source,
    /// so all empathy, signals, and visible-card information reflect what `player_index` knows
    /// (as tracked by the current observer), not what the current observer knows.
    fn as_player_pov(&self, player_index: PlayerIndex) -> LightweightPlayerPOV<'_> {
        LightweightPlayerPOV::new(
            player_index,
            self.team_knowledge().player(player_index),
            self.team_knowledge(),
            self.table_state(),
            self.static_data(),
        )
    }

    /// Returns true if the card is known to be trash from this POV
    fn is_known_trash(&self, card_deck_index: CardDeckIndex) -> bool;

    /// Best mask of possible identities for a card from this player's POV.
    ///
    /// Combines three layers:
    /// - public clue empathy (baseline)
    /// - convention-derived narrowing (GTP, clue interpretation, live hypotheses)
    /// - observable narrowing — only applied to the player's own (unseen) cards:
    ///   identities whose copies are all accounted for in visible positions
    ///   (other players' hands + played stacks + discard pile) are excluded.
    ///
    /// Cards in other players' hands skip the observable step: their identity is
    /// already pinned to a visible singleton by `combined_possible_identities`, and
    /// the observable mask counts those same copies, which would spuriously contradict.
    ///
    /// On contradiction between convention-narrowing and observable, falls back to
    /// `baseline ∩ observable` (or `observable` alone if that too is empty) — the
    /// convention reasoning is dropped but public clue constraints are preserved.
    fn inferred_identities(&self, card_deck_index: CardDeckIndex) -> CardIdentityMask;

    fn valid_actions(&self) -> Vec<GameAction>;

    fn gotten_cards(&self) -> CardIdentityMask;

    fn is_gotten(&self, variant_card_id: VariantCardId) -> bool;
}
