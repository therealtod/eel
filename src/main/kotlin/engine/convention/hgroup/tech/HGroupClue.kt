package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

abstract class HGroupClue(override val name: String) : HGroupTech(), ClueTech {
    abstract fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean

    abstract fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PlayerPersonalKnowledge

    protected fun getFocusedSlot(
        hand: Hand,
        clueValue: ClueValue,
        playerPOV: PlayerPOV,
    ): Slot {
        val touchedSlotsIndexes = hand.getSlotsTouchedBy(clueValue)
        return getFocusedSlot(touchedSlotsIndexes, hand, playerPOV)
    }

    protected fun getFocusedSlot(
        touchedSlotsIndexes: Set<Int>,
        hand: Hand,
        playerPOV: PlayerPOV,
    ): Slot {
        require(touchedSlotsIndexes.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        val touchedSlots = touchedSlotsIndexes.map {hand.getSlot(it) }
        if (hasChop(hand, playerPOV)) {
            val chop = getChop(hand, playerPOV)
            return if (touchedSlotsIndexes.contains(chop.index)) {
                chop
            } else {
                (touchedSlots.firstOrNull { !isSlotTouched(it.index, hand, playerPOV) } ?: touchedSlots.first())
            }
        } else {
            return touchedSlots.first()
        }
    }

    protected fun getFocusedSlotIndex(
        hand: Hand,
        touchedSlotsIndexes: Set<Int>,
        playerPOV: PlayerPOV,
    ): Int {
        return getFocusedSlot(
            touchedSlotsIndexes = touchedSlotsIndexes,
            hand = hand,
            playerPOV = playerPOV,
        ).index
    }

    protected fun getAllCluesFocusing(
        slotIndex: Int,
        teammate: VisibleTeammate,
        playerPOV: PlayerPOV,
    ): Set<ClueAction> {
        val card = teammate.getCardInSlot(slotIndex)
        val ranks = playerPOV.globallyAvailableInfo.getCluableRanks().filter { card.isTouchedBy(it) }
        val colors = playerPOV.globallyAvailableInfo.getCluableColors().filter { card.isTouchedBy(it) }
        val clueValues = (ranks + colors).filter {
            getFocusedSlot(
                hand = teammate.hand,
                clueValue = it,
                playerPOV = playerPOV

            ).index == slotIndex
        }

        return clueValues.map {
            GameActionFactory.createClueAction(
                clueGiver = playerPOV.getOwnPlayerId(),
                clueReceiver = teammate.playerId,
                clueValue = it
            )
        }.toSet()
    }

    open fun matchesClueBySlot(focusIndex: Int, hand: Hand, playerPOV: PlayerPOV): Boolean {
        return true
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes,
            playerPOV = playerPOV,
        )
        if (!matchesClueBySlot(focusIndex, receiverHand, playerPOV)) {
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

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
        val receiverId = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(receiverId)
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = action.slotsTouched,
            playerPOV = playerPOV,
            )
        return getGeneratedKnowledge(action, focusIndex, playerPOV)
    }
}
