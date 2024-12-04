package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.action.ClueAction

abstract class HGroupClue(override val name: String) : HGroupTech(), ClueTech {
    abstract fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean

    abstract fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PersonalKnowledge

    protected fun getFocusedSlot(
        hand: InterpretedHand,
        clueValue: ClueValue,
        playerPOV: PlayerPOV,
    ): InterpretedSlot {
        val touchedSlots = hand.getSlotsTouchedBy(clueValue, playerPOV)
        return getFocusedSlot(hand, touchedSlots.map { it.index }.toSet())
    }

    protected fun getFocusedSlot(
        hand: VisibleHand,
        clueValue: ClueValue,
        playerPOV: PlayerPOV,
    ): InterpretedSlot {
        val touchedSlotsIndexes = hand.getSlotsTouchedBy(clueValue, playerPOV)
        return getFocusedSlot(hand, touchedSlotsIndexes.map { it.index }.toSet())
    }

    protected fun getFocusedSlot(
        hand: InterpretedHand,
        touchedSlotsIndexes: Set<Int>,
    ): InterpretedSlot {
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

    protected fun getFocusedSlotIndex(
        hand: InterpretedHand,
        touchedSlotsIndexes: Set<Int>,
    ): Int {
        return getFocusedSlot(
            hand = hand,
            touchedSlotsIndexes = touchedSlotsIndexes
        ).index
    }

    protected fun getAllCluesFocusing(
        slotIndex: Int,
        teammate: VisibleTeammate,
        playerPOV: PlayerPOV,
    ): Set<ClueAction> {
        val card = teammate.getVisibleHand().getCardinSlot(slotIndex)
        val ranks = playerPOV.globallyAvailableInfo.getCluableRanks().filter { card.isTouchedBy(it) }
        val colors = playerPOV.globallyAvailableInfo.getCluableColors().filter { card.isTouchedBy(it) }
        val clueValues = (ranks + colors).filter {
            getFocusedSlot(
                hand = teammate.getVisibleHand(),
                clueValue = it,
                playerPOV = playerPOV
            ).contains(card)
        }

        return clueValues.map {
            GameActionFactory.createClueAction(
                clueGiver = playerPOV.getOwnPlayerId(),
                clueReceiver = teammate.playerId,
                clueValue = it
            )
        }.toSet()
    }

    open fun matchesClueBySlot(focusIndex: Int, hand: InterpretedHand): Boolean {
        return true
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        if (!matchesClueBySlot(focusIndex, receiverHand)) {
            return false
        }
        if (clueReceiver != playerPOV.getOwnPlayerId()) {
            val teammate = playerPOV.getTeammate(clueReceiver).asVisible()
            return teammateSlotMatchesCondition(
                teammate = teammate,
                slotIndex = focusIndex,
                playerPOV = playerPOV
            )
        } else {
            return matchesReceivedClue(
                clue = action,
                focusIndex = focusIndex,
                playerPOV = playerPOV,
            )
        }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PersonalKnowledge {
        val receiverId = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(receiverId)
        val focusIndex = getFocusedSlotIndex(hand = receiverHand, touchedSlotsIndexes = action.slotsTouched)
        return getGeneratedKnowledge(action, focusIndex, playerPOV)
    }
}
