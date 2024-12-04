package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

abstract class HGroupTech : ConventionTech {
    abstract fun appliesTo(card: HanabiCard, variant: Variant): Boolean

    /*

    fun getFocusedSlot(
        hand: InterpretedHand,
        clueValue: ClueValue,
    ): Slot {
        val touchedSlotsIndexes = hand.getSlotsTouchedBy(clueValue)
        return getFocusedSlot(hand, touchedSlotsIndexes.map { it.index }.toSet())
    }

    fun getFocusedSlot(
        hand: InterpretedHand,
        touchedSlotsIndexes: Set<Int>,
    ): Slot {
        require(touchedSlotsIndexes.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        val touchedSlots = touchedSlotsIndexes.map { hand.getSlot(it) }
        if (hasChop(hand)) {
            val chop = getChop(hand)
            return if (touchedSlotsIndexes.contains(chop.index)) {
                chop
            } else {
                (touchedSlots.firstOrNull { !it.isTouched() } ?: touchedSlots.first())
            }
        } else {
            return touchedSlots.first()
        }
    }

     */

    fun isGloballyKnownPlayable(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        val prerequisiteCards = card.getPrerequisiteCards()
        val playedCardsForSuite = playerPOV.globallyAvailableInfo.getStackForCard(card).cards
        val teammatesKnownCards = playerPOV.getTeammates().flatMap { it.getOwnKnownCards() }
        val ownKnownCards = playerPOV.getOwnKnownCards()
        return (playedCardsForSuite + teammatesKnownCards + ownKnownCards).containsAll(prerequisiteCards)
    }

    fun hasChop(hand: InterpretedHand): Boolean {
        return hand.any { !it.isTouched() }
    }

    fun getChop(hand: InterpretedHand): InterpretedSlot {
        return hand.last { !it.isTouched() }
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        return false
    }
}
