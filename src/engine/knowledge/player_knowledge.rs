use smallvec::SmallVec;

use crate::engine::convention::hgroup::signal::Signal;
use crate::engine::knowledge::knowledge_update::{
    Hypothesis, HypothesisId, KnowledgeUpdate, PendingTrigger, TrackedHypothesis,
};
use crate::game::MAX_CARDS_IN_DECK;
use crate::game::action::game_action::GameAction;
use crate::game::card::{
    CardDeckIndex, CardIdentityMask, DeckCardsBitField, VariantCardId, VariantCardsBitField,
};
use crate::game::state::PlayerIndex;
use crate::game::state::table_state::TableState;
use crate::game::variant::Variant;

/// Per-player mutable knowledge storage.
///
/// Tracks what a specific player knows (or can infer) about every card in the deck.
/// One instance per player, stored inside [`TeamKnowledge`](super::team_knowledge::TeamKnowledge).
///
/// # Knowledge layers
///
/// - **Baseline** (`inferred_identities`, `signals`): unconditional facts. Comes
///   from revealed cards, manual scenario setup, and confirmed hypotheses that have
///   been baked in.
/// - **Hypothesis cohorts** (`hypotheses`): tech-derived interpretations of observed
///   actions. Each hypothesis is one tech's claim; hypotheses sharing a `cohort_id`
///   come from the same observed action. The effective narrowing on any card is
///   the **union** of cohort hypothesis masks targeting that card, intersected with
///   baseline.
///
/// Use [`effective_inferred_mask`](Self::effective_inferred_mask) and
/// [`has_play_signal`](Self::has_play_signal) to read effective state — they
/// combine baseline with live hypothesis contributions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerKnowledge {
    /// Index of the player this knowledge belongs to.
    pub player_index: usize,
    /// Baseline narrowing of card identities. Hypothesis contributions are NOT
    /// baked in here until they confirm.
    pub inferred_identities: [Option<CardIdentityMask>; MAX_CARDS_IN_DECK],
    /// Baseline signals. Hypothesis-contributed signals live in `hypotheses`.
    /// `SmallVec<[Signal; 2]>` avoids heap allocation for the common 0–2 signals case.
    pub signals: [SmallVec<[Signal; 2]>; MAX_CARDS_IN_DECK],
    /// Bitfield of cards whose identity is visible to this player.
    pub visible_cards: DeckCardsBitField,
    /// Bitfield of cards currently in this player's own hand.
    pub own_hand: DeckCardsBitField,
    /// Tracked hypotheses, flat. Hypotheses sharing the same `cohort_id` are
    /// siblings (interpretations of the same observed action).
    pub hypotheses: Vec<TrackedHypothesis>,
}

impl PlayerKnowledge {
    /// Create a new `PlayerKnowledgeState` with no knowledge.
    pub fn new(player_index: usize) -> Self {
        PlayerKnowledge {
            player_index,
            inferred_identities: [None; MAX_CARDS_IN_DECK],
            signals: std::array::from_fn(|_| SmallVec::new()),
            visible_cards: 0,
            own_hand: 0,
            hypotheses: Vec::new(),
        }
    }

    /// Create a default (empty) state, used for padding fixed-size arrays.
    pub fn empty() -> Self {
        Self::new(0)
    }

    /// Mark a card as revealed (visible) and set its identity.
    pub fn update_with_revealed_card(
        &mut self,
        card_deck_index: CardDeckIndex,
        card_id: VariantCardId,
    ) {
        let idx = card_deck_index as usize;
        self.inferred_identities[idx] = Some(CardIdentityMask::known(card_id));
        self.visible_cards |= 1 << card_deck_index;
    }

    /// Restrict the possible identities of a card to only those in the given mask
    /// in **baseline**. Used by non-hypothesis paths (revealed cards, direct
    /// scenario setup, hypothesis confirmation that bakes in the survivor).
    pub fn narrow_inferred(
        &mut self,
        card_deck_index: CardDeckIndex,
        mask: VariantCardsBitField,
        variant: &Variant,
    ) {
        let idx = card_deck_index as usize;
        let current =
            self.inferred_identities[idx].unwrap_or_else(|| CardIdentityMask::all(variant));
        if let Some(new_empathy) = current.narrow(mask) {
            self.inferred_identities[idx] = Some(new_empathy);
            if new_empathy.is_exactly_known() {
                self.visible_cards |= 1 << card_deck_index;
            }
        }
    }

    /// Attach a baseline signal to a card.
    pub fn add_signal(&mut self, card_deck_index: CardDeckIndex, signal: Signal) {
        self.signals[card_deck_index as usize].push(signal);
    }

    // ── Hypothesis lifecycle ────────────────────────────────────────────────

    /// Apply a single non-hypothesis [`KnowledgeUpdate`] to baseline. Used by
    /// hypothesis confirmation (baking the survivor) and direct scenario writes.
    fn apply_baseline_update(&mut self, update: &KnowledgeUpdate, variant: &Variant) {
        match update {
            KnowledgeUpdate::NarrowPossibilities {
                card_deck_index,
                mask,
            } => self.narrow_inferred(*card_deck_index, *mask, variant),
            KnowledgeUpdate::AddSignal {
                card_deck_index,
                signal,
            } => self.add_signal(*card_deck_index, signal.clone()),
        }
    }

    /// Store a cohort of hypotheses produced by interpreting one observed action.
    ///
    /// `cohort_id` is shared by all hypotheses in the cohort. `next_id` is a
    /// dispatcher-supplied counter used to assign each hypothesis a unique
    /// `HypothesisId`; it is incremented as hypotheses are stored.
    ///
    /// **Single-hypothesis-without-trigger optimization**: if `hypotheses` contains
    /// exactly one hypothesis with no trigger, it is baked directly into baseline
    /// rather than stored as a cohort entry. This preserves prior behavior for the
    /// common case of an unambiguous interpretation.
    pub fn apply_cohort(
        &mut self,
        cohort_id: HypothesisId,
        hypotheses: Vec<Hypothesis>,
        next_id: &mut HypothesisId,
        variant: &Variant,
    ) {
        let non_empty: Vec<_> = hypotheses.into_iter().filter(|h| !h.is_empty()).collect();
        if non_empty.is_empty() {
            return;
        }
        // Bake unambiguous unconditional hypotheses into baseline.
        if non_empty.len() == 1 && non_empty[0].trigger.is_none() {
            for u in &non_empty[0].immediate {
                self.apply_baseline_update(u, variant);
            }
            return;
        }
        for h in non_empty {
            let id = *next_id;
            *next_id += 1;
            self.hypotheses.push(TrackedHypothesis {
                id,
                cohort_id,
                immediate: h.immediate,
                trigger: h.trigger,
            });
        }
    }

    /// Resolve any provisional hypotheses triggered by `actor`'s observed `action`.
    ///
    /// For each hypothesis whose trigger matches the action:
    /// - **Confirm**: the hypothesis survives. Its narrowings/signals are baked
    ///   into baseline; **all sibling hypotheses in the same cohort are dropped**.
    /// - **Reject**: the hypothesis is dropped from the list. Sibling hypotheses
    ///   in the same cohort remain (the receiver's superposition narrows by one
    ///   branch).
    pub fn resolve_pending(&mut self, actor: PlayerIndex, action: &GameAction, variant: &Variant) {
        if self.hypotheses.is_empty() {
            return;
        }
        let mut confirmed_cohorts: Vec<HypothesisId> = Vec::new();
        let mut rejected_ids: Vec<HypothesisId> = Vec::new();
        // Snapshot id+cohort+trigger to decide outcomes without holding a borrow.
        let triggers: Vec<(HypothesisId, HypothesisId, PendingTrigger)> = self
            .hypotheses
            .iter()
            .filter_map(|h| h.trigger.clone().map(|t| (h.id, h.cohort_id, t)))
            .collect();
        for (id, cohort_id, trigger) in triggers {
            match trigger {
                PendingTrigger::BlindPlay {
                    player,
                    expected_card,
                    ..
                } if player == actor => {
                    let confirmed = matches!(
                        action,
                        GameAction::Play {
                            card_deck_index,
                            ..
                        } if *card_deck_index == expected_card
                    );
                    if confirmed {
                        confirmed_cohorts.push(cohort_id);
                        // Bake this hypothesis's updates into baseline before pruning siblings.
                        if let Some(h) = self.hypotheses.iter().find(|h| h.id == id) {
                            let updates = h.immediate.clone();
                            for u in &updates {
                                self.apply_baseline_update(u, variant);
                            }
                        }
                    } else {
                        rejected_ids.push(id);
                    }
                }
                _ => {}
            }
        }
        if !confirmed_cohorts.is_empty() || !rejected_ids.is_empty() {
            self.hypotheses.retain(|h| {
                !confirmed_cohorts.contains(&h.cohort_id) && !rejected_ids.contains(&h.id)
            });
        }
    }

    // ── Effective state accessors ───────────────────────────────────────────

    /// Effective inferred mask for a card, combining baseline with hypothesis
    /// contributions.
    ///
    /// For each cohort that touches this card, the contributions are *unioned*
    /// (the card could be any of the hypothesis interpretations). Across cohorts
    /// (and against baseline), masks are *intersected*.
    pub fn effective_inferred_mask(
        &self,
        card_deck_index: CardDeckIndex,
        variant: &Variant,
    ) -> CardIdentityMask {
        let baseline = self.inferred_identities[card_deck_index as usize]
            .unwrap_or_else(|| CardIdentityMask::all(variant))
            .as_bits();
        let mut mask = baseline;
        // Group hypotheses by cohort, union per cohort, intersect across cohorts.
        let mut visited_cohorts: Vec<HypothesisId> = Vec::new();
        for h in &self.hypotheses {
            if visited_cohorts.contains(&h.cohort_id) {
                continue;
            }
            visited_cohorts.push(h.cohort_id);
            let mut cohort_union: u64 = 0;
            let mut cohort_touches_card = false;
            for sibling in self
                .hypotheses
                .iter()
                .filter(|s| s.cohort_id == h.cohort_id)
            {
                for u in &sibling.immediate {
                    if let KnowledgeUpdate::NarrowPossibilities {
                        card_deck_index: idx,
                        mask: m,
                    } = u
                    {
                        if *idx == card_deck_index {
                            cohort_union |= *m;
                            cohort_touches_card = true;
                        }
                    }
                }
            }
            if cohort_touches_card {
                mask &= cohort_union;
            }
        }
        CardIdentityMask::from_bits(mask)
    }

    /// True if any baseline OR live hypothesis attaches a [`Signal::Play`] to the card.
    pub fn has_play_signal(&self, card_deck_index: CardDeckIndex) -> bool {
        if self.signals[card_deck_index as usize]
            .iter()
            .any(|s| matches!(s, Signal::Play { .. }))
        {
            return true;
        }
        self.hypotheses.iter().any(|h| {
            h.immediate.iter().any(|u| {
                matches!(
                    u,
                    KnowledgeUpdate::AddSignal {
                        card_deck_index: idx,
                        signal: Signal::Play { .. },
                    } if *idx == card_deck_index
                )
            })
        })
    }

    /// Get the possible identities for a card from baseline only (does not include
    /// live hypothesis contributions). For effective state combining baseline with
    /// hypotheses, use [`effective_inferred_mask`](Self::effective_inferred_mask).
    pub fn possible_identities(&self, card_deck_index: CardDeckIndex) -> Option<CardIdentityMask> {
        self.inferred_identities[card_deck_index as usize]
    }

    /// Get the combined knowledge for a card: game-rule empathy merged with
    /// effective inferred identities (baseline + live hypotheses).
    ///
    /// For cards in the player's own hand that haven't been identified by
    /// convention (i.e. not in `visible_cards`), only convention-inferred knowledge
    /// is returned. The player cannot see their own cards, so the omniscient deck
    /// empathy must not leak into their decision-making during search.
    pub fn combined_possible_identities(
        &self,
        card_deck_index: CardDeckIndex,
        table_state: &TableState,
        variant: &Variant,
    ) -> CardIdentityMask {
        let is_own_unseen = (self.own_hand >> card_deck_index) & 1 != 0
            && (self.visible_cards >> card_deck_index) & 1 == 0;

        let effective = self.effective_inferred_mask(card_deck_index, variant);

        if is_own_unseen {
            return effective;
        }

        let game_empathy = table_state.deck.get_global_empathy(card_deck_index);
        if let Some(combined) = game_empathy.narrow(effective.as_bits()) {
            combined
        } else {
            game_empathy
        }
    }
}

#[cfg(test)]
pub fn knowledge_with_visible(player_index: usize, visible: &[(u8, u64)]) -> PlayerKnowledge {
    let mut k = PlayerKnowledge::new(player_index);
    for &(idx, mask) in visible {
        k.inferred_identities[idx as usize] = Some(CardIdentityMask::from_bits(mask));
        k.visible_cards |= 1 << idx;
    }
    k
}

#[cfg(any(test, feature = "test-support"))]
pub fn knowledge_for_hand(cards: &[u8]) -> PlayerKnowledge {
    let mut k = PlayerKnowledge::new(0);
    for &idx in cards {
        k.own_hand |= 1 << idx;
    }
    k
}

#[cfg(test)]
pub fn knowledge_with_empathy(
    card_deck_index: CardDeckIndex,
    possible_identities: VariantCardsBitField,
) -> PlayerKnowledge {
    let mut k = PlayerKnowledge::new(0);
    k.inferred_identities[card_deck_index as usize] =
        Some(CardIdentityMask::from_bits(possible_identities));
    k.own_hand = 1 << card_deck_index;
    k
}
