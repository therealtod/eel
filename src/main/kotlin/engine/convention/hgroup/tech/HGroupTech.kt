package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.ColorClueAction
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
                playerPOV = playerPOV,
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

    override fun overrides(otherTech: ConventionTech<T>): Boolean {
        TODO("Not yet implemented")
    }

    fun getFocusedSlot(
        playerPOV: PlayerPOV,
        hand: InterpretedHand,
        clueValue: ClueValue,
    ): Slot {
        val touchedSlots = hand.getSlotsTouchedBy(clueValue)
        require(touchedSlots.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        if (hasChop(hand)) {
            val chop = getChop(hand)
            return if (touchedSlots.contains(chop)) {
                chop
            } else {
                (touchedSlots.firstOrNull { !it.isTouched() } ?: touchedSlots.first())
            }
        } else {
            return touchedSlots.first()
        }
    }

    override fun toString() = name
}

