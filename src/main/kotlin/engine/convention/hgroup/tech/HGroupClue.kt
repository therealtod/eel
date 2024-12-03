package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

abstract class HGroupClue(override val name: String) : HGroupTech, ClueTech() {
    abstract fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean

    abstract fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PersonalKnowledge

    protected fun getFocusedSlot(
        hand: InterpretedHand,
        clueValue: ClueValue,
        playerPOV: PlayerPOV,
    ): InterpretedSlot {
        val touchedSlots = hand.getSlotsTouchedBy(clueValue, playerPOV)
        return HGroupCommon.getFocusedSlot(hand, touchedSlots.map { it.index }.toSet())
    }

    protected fun getFocusedSlot(
        hand: InterpretedHand,
        touchedSlotsIndexes: Set<Int>,
    ): Slot {
        return HGroupCommon.getFocusedSlot(hand, touchedSlotsIndexes)
    }

    protected fun getFocusedSlotIndex(
        hand: VisibleHand,
        clueValue: ClueValue,
    ): Int {

        return HGroupCommon.getFocusedSlot(
            hand = hand,
            clueValue = clueValue,
        ).index
    }

    protected fun getFocusedSlotIndex(
        hand: InterpretedHand,
        touchedSlotsIndexes: Set<Int>,
    ): Int {
        return HGroupCommon.getFocusedSlot(
            hand = hand,
            touchedSlotsIndexes = touchedSlotsIndexes
        ).index
    }

    protected fun getAllFocusingClues(
        playerPOV: PlayerPOV,
        slot: VisibleSlot,
        teammate: Teammate,
    ): Set<ClueAction> {
        val card = slot.card
        val ranks = playerPOV.globallyAvailableInfo.getCluableRanks().filter { card.isTouchedBy(it) }
        val colors = playerPOV.globallyAvailableInfo.getCluableColors().filter { card.isTouchedBy(it) }
        val clueValues = (ranks + colors).filter {
            getFocusedSlot(
                hand = teammate.hand,
                clueValue = it
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
            val teammate = playerPOV.getTeammate(clueReceiver)
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
