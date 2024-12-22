package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

abstract class HGroupClue(override val name: String) : HGroupTech(), ClueTech {
    override fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: GameFromPlayerPOV): Boolean {
        val clueReceiverId = clueAction.clueReceiver
        val clueReceiverPOV = playerPOV.getPlayerPOV(clueReceiverId)
        val receiverHand = clueReceiverPOV.getOwnHand()
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotsIndexes,
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
                clueAction = clueAction,
                touchedSlotsIndexes = touchedSlotsIndexes,
                focusIndex = focusIndex,
                playerPOV = playerPOV,
            )
        }
    }

    abstract fun matchesReceivedClue(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV,
    ):  Boolean

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        playerPOV: GameFromPlayerPOV
    ): Knowledge {
        val clueReceiverId = clueAction.clueReceiver
        val clueReceiverPOV = playerPOV.getPlayerPOV(clueReceiverId)
        val receiverHand = clueReceiverPOV.getOwnHand()
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotsIndexes,
            playerPOV = playerPOV,
        )
        return getGeneratedKnowledge(
            clueAction = clueAction,
            touchedSlotsIndexes = touchedSlotsIndexes,
            focusIndex = focusIndex,
            playerPOV = playerPOV,
        )
    }

    abstract fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV,
    ): Knowledge

    protected fun getFocusedSlot(
        hand: Hand,
        clueValue: ClueValue,
        playerPOV: GameFromPlayerPOV,
    ): Slot {
        val touchedSlotsIndexes = hand.getSlotsTouchedBy(clueValue)
        return getFocusedSlot(touchedSlotsIndexes, hand, playerPOV)
    }

    protected fun getFocusedSlot(
        touchedSlotsIndexes: Set<Int>,
        hand: Hand,
        playerPOV: GameFromPlayerPOV,
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
        playerPOV: GameFromPlayerPOV,
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
        playerPOV: GameFromPlayerPOV,
    ): Set<ClueAction> {
        val possibleClues = playerPOV.getGameData().getAvailableClueValues()
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

    open fun matchesClueBySlot(focusIndex: Int, hand: Hand, playerPOV: GameFromPlayerPOV): Boolean {
        return true
    }
}
