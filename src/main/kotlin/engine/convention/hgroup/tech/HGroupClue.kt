package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

abstract class HGroupClue(override val name: String) : HGroupTech(), ClueTech {
    abstract fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean

    abstract fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Knowledge

    protected fun getFocusedSlot(
        hand: Hand,
        clueValue: ClueValue,
        activePlayer: ActivePlayer,
    ): Slot {
        val touchedSlotsIndexes = hand.getSlotsTouchedBy(clueValue)
        return getFocusedSlot(touchedSlotsIndexes, hand, activePlayer)
    }

    protected fun getFocusedSlot(
        touchedSlotsIndexes: Set<Int>,
        hand: Hand,
        activePlayer: ActivePlayer,
    ): Slot {
        require(touchedSlotsIndexes.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        val touchedSlots = touchedSlotsIndexes.map {hand.getSlot(it) }
        if (hasChop(hand, activePlayer)) {
            val chop = getChop(hand, activePlayer)
            return if (touchedSlotsIndexes.contains(chop.index)) {
                chop
            } else {
                (touchedSlots.firstOrNull { !isSlotTouched(it.index, hand, activePlayer) } ?: touchedSlots.first())
            }
        } else {
            return touchedSlots.first()
        }
    }

    protected fun getFocusedSlotIndex(
        hand: Hand,
        touchedSlotsIndexes: Set<Int>,
        activePlayer: ActivePlayer,
    ): Int {
        return getFocusedSlot(
            touchedSlotsIndexes = touchedSlotsIndexes,
            hand = hand,
            activePlayer = activePlayer,
        ).index
    }

    protected fun getAllCluesFocusing(
        slot: Slot,
        engineHandlerPlayer: EngineHandlerPlayer,
        activePlayer: ActivePlayer,
    ): Set<ClueAction> {
        val possibleClues = activePlayer.globallyAvailableInfo.getAvailableClueValues()
        val cluesTouchingSlot = possibleClues
            .filter { clueValue ->
                slot.isTouchedBy(clueValue)
            }
        val clueValuesWithCorrectFocus = cluesTouchingSlot.filter {
            getFocusedSlot(
                hand = engineHandlerPlayer.hand,
                clueValue = it,
                activePlayer = activePlayer

            ).index == slot.index
        }

        return clueValuesWithCorrectFocus.map {
            GameActionFactory.createClueAction(
                clueGiver = activePlayer.getOwnPlayerId(),
                clueReceiver = engineHandlerPlayer.playerId,
                clueValue = it
            )
        }.toSet()
    }

    open fun matchesClueBySlot(focusIndex: Int, hand: Hand, activePlayer: ActivePlayer): Boolean {
        return true
    }

    override fun matchesClue(action: ObservedClue, activePlayer: ActivePlayer): Boolean {
        val clueReceiverId = action.clueAction.clueReceiver
        val clueReceiverPOV = activePlayer.getPlayerPOV(clueReceiverId)
        val receiverHand = clueReceiverPOV.getOwnHand()
        val touchedSlotIndexes = action.slotsTouched
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes,
            activePlayer = activePlayer,
        )
        if (!matchesClueBySlot(focusIndex, receiverHand, activePlayer)) {
            return false
        }
        if (clueReceiverId != activePlayer.getOwnPlayerId()) {
            val teammate = activePlayer.getTeammate(clueReceiverId)
            return teammateSlotMatchesCondition(
                engineHandlerPlayer = teammate,
                slot = teammate.hand.getSlot(focusIndex),
                activePlayer = activePlayer
            )
        } else {
            return matchesReceivedClue(
                clue = action,
                focusIndex = focusIndex,
                activePlayer = activePlayer,
            )
        }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, activePlayer: ActivePlayer): Knowledge {
        val receiverId = action.clueAction.clueReceiver
        val receiverPOV = activePlayer.getTeammate(receiverId).getPOV(activePlayer)
        val receiverHand = receiverPOV.getOwnHand()
        val focusIndex = getFocusedSlotIndex(
            hand = receiverHand,
            touchedSlotsIndexes = action.slotsTouched,
            activePlayer = activePlayer,
            )
        return getGeneratedKnowledge(action, focusIndex, activePlayer)
    }
}
