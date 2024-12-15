package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

abstract class HGroupClue(override val name: String) : HGroupTech(), ClueTech {
    abstract fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean

    abstract fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Knowledge

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
        slot: Slot,
        teammate: Teammate,
        playerPOV: PlayerPOV,
    ): Set<ClueAction> {
        val possibleClues = playerPOV.globallyAvailableInfo.getAvailableClueValues()
        val cluesTouchingSlot = possibleClues
            .filter { clueValue ->
                slot.isTouchedBy(clueValue)
            }
        val clueValuesWithCorrectFocus = cluesTouchingSlot.filter {
            getFocusedSlot(
                hand = teammate.hand,
                clueValue = it,
                playerPOV = playerPOV

            ).index == slot.index
        }

        return clueValuesWithCorrectFocus.map {
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
        val clueReceiverId = action.clueAction.clueReceiver
        val clueReceiverPOV = playerPOV.getPlayerPOV(clueReceiverId)
        val receiverHand = clueReceiverPOV.getOwnHand()
        val touchedSlotIndexes = action.slotsTouched
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes,
            playerPOV = playerPOV,
        )
        if (!matchesClueBySlot(focusIndex, receiverHand, playerPOV)) {
            return false
        }
        if (clueReceiverId != playerPOV.getOwnPlayerId()) {
            val teammate = playerPOV.getTeammate(clueReceiverId)
            return teammateSlotMatchesCondition(
                teammate = teammate,
                slot = teammate.hand.getSlot(focusIndex),
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

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): Knowledge {
        val receiverId = action.clueAction.clueReceiver
        val receiverPOV = playerPOV.getTeammate(receiverId).getPOV(playerPOV)
        val receiverHand = receiverPOV.getOwnHand()
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = action.slotsTouched,
            playerPOV = playerPOV,
            )
        return getGeneratedKnowledge(action, focusIndex, playerPOV)
    }
}
