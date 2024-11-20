package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
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
        teammate: Teammate,
    ): Set<GiveClue> {
        val hand = teammate.hand
        val ranks = card.getRanksTouchingCard()
        val colors = card.getColorsTouchingCard()
        val clues = (ranks.map {
            RankClue(rank = it)
        } + colors.map {
            ColorClue(color = it)
        }).filter {
            val focusSlotIndex = HGroupCommon.getClueFocusSlotIndex(clue = it, hand = hand)
            teammate.getCardAtSlot(focusSlotIndex) == card
        }

        return clues.map {
            GiveClue(
                clue = it,
                to = teammate.playerId,
            )
        }.toSet()
    }

    private fun getRankCluesFocusing(
        slot: Slot,
        hand: InterpretedHand,
        ranks: Set<Rank>,
    ): Set<RankClue>{
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

