package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.Clue
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import eelst.ilike.game.entity.card.HanabiCard

abstract class HGroupTech(
    override val name: String,
    private val takesPrecedenceOver: Set<HGroupTech>,
) : ConventionTech {
    protected fun getAllFocusingClues(
        card: HanabiCard,
        slot: Slot,
        hand: InterpretedHand,
    ): Set<Clue> {
        val ranks = card.getRanksTouchingCard()
        val colors = card.getColorsTouchingCard()
        return getRankCluesFocusing(
            slot = slot,
            hand = hand,
            ranks = ranks,
        ) +
                getColorCluesFocusing(
                    slot = slot,
                    hand = hand,
                    colors = colors,
                )
    }

    protected fun getAllFocusingActions(
        card: HanabiCard,
        slot: Slot,
        hand: InterpretedHand,
    ): Set<ConventionalAction> {
        val clues = getAllFocusingClues(
            card = card,
            slot = slot,
            hand = hand,
        )
        return clues.map {
            ConventionalAction(
                action = TODO(),
                tech = this
            )
        }.toSet()
    }

    private fun getRankCluesFocusing(
        slot: Slot,
        hand: InterpretedHand,
        ranks: Set<Rank>,
    ): Set<RankClue> {
        return ranks.map {
            RankClue(rank = it)
        }
            .filter { HGroupCommon.getClueFocusSlotIndex(clue = it, hand = hand) == slot.index }
            .toSet()
    }

    private fun getColorCluesFocusing(
        slot: Slot,
        hand: InterpretedHand,
        colors: Set<Color>,
    ): Set<ColorClue> {
        return colors.map {
            ColorClue(color = it)
        }
            .filter { HGroupCommon.getClueFocusSlotIndex(clue = it, hand) == slot.index }
            .toSet()
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        return takesPrecedenceOver.contains(otherTech)
    }

    override fun toString() = name
}

