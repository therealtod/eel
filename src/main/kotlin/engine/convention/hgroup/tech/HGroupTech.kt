package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
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

    fun isGloballyKnownPlayable(card: HanabiCard, activePlayer: ActivePlayer): Boolean {
        val prerequisiteCards = card.getPrerequisiteCards()
        val playedCardsForSuite = activePlayer.globallyAvailableInfo.getStackForCard(card).cards
        val teammatesKnownCards = activePlayer.getTeammates().flatMap { it.getPOV(activePlayer).getOwnKnownCards() }
        val ownKnownCards = activePlayer.getOwnKnownCards()
        return (playedCardsForSuite + teammatesKnownCards + ownKnownCards).containsAll(prerequisiteCards)
    }

    fun hasChop(hand: Hand, activePlayer: ActivePlayer): Boolean {
        return hand.any { !isSlotTouched(it.index, hand, activePlayer) }
    }

    fun getChop(hand: Hand, activePlayer: ActivePlayer): Slot {
        return hand.last { !isSlotTouched(it.index, hand, activePlayer) }
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        return false
    }

    protected fun isSlotTouched(slotIndex: Int, hand: Hand, activePlayer: ActivePlayer): Boolean {
        return hand.getSlot(slotIndex).isTouched()
    }
}
