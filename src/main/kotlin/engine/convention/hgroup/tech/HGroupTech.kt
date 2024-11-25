package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.card.HanabiCard

abstract class HGroupTech<T: GameAction>(
    override val name: String,
    private val takesPrecedenceOver: Set<HGroupTech<T>> = emptySet(),
) : ConventionTech<T> {
    protected fun getAllFocusingClues(
        playerPOV: PlayerPOV,
        card: HanabiCard,
        teammate: Teammate,
    ): Set<ClueAction> {
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

    override fun overrides(otherTech: ConventionTech<T>): Boolean {
        return false
    }

    override fun matches(action: ObservedAction<T>, playerPOV: PlayerPOV): Boolean {
        return when(action) {
            is ObservedPlay -> matchesPlay(action, playerPOV)
            is ObservedDiscard -> matchesDiscard(action, playerPOV)
            is ObservedClue -> matchesClue(action, playerPOV)
        }
    }

    override fun matchesPlay(action: ObservedPlay, playerPOV: PlayerPOV): Boolean {
        return false
    }

    override fun matchesDiscard(action: ObservedDiscard, playerPOV: PlayerPOV): Boolean {
        return false
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        return false
    }

    override fun toString() = name
}

