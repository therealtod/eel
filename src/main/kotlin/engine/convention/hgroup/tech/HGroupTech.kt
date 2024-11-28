package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant

interface HGroupTech: ConventionTech {
    fun appliesTo(card: HanabiCard, variant: Variant): Boolean

    fun getAllFocusingClues(
        playerPOV: PlayerPOV,
        slot: VisibleSlot,
        teammate: Teammate,
    ): Set<ClueAction> {
        val card = slot.card
        val ranks = card.getRanksTouchingCard()
        val colors = card.getColorsTouchingCard()
        val clueValues = (ranks + colors).filter {
            getFocusedSlot(
                hand = teammate.hand,
                clueValue = it
            ).contains(card)
        }

        return clueValues.map {
            GameActionFactory.createClueAction(
                clueGiver = playerPOV.playerId,
                clueReceiver = teammate.playerId,
                clueValue = it
            )
        }.toSet()
    }

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

    override fun overrides(otherTech: ConventionTech): Boolean {
        return false
    }
}
