package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.suite.*

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableSetOf<ClueAction>()

        playerPOV.forEachTeammate { teammate ->
            if (hasChop(teammate.hand)) {
                val chop = getChop(teammate.hand)
                val card = teammate.getCardAtSlot(chop.index)
                if (appliesTo.contains(card.suite) &&
                    card.rank != Rank.FIVE &&
                    playerPOV.globallyAvailableInfo.isCritical(card) &&
                    !isGloballyKnownPlayable(card, playerPOV)
                ) {
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            card = card,
                            teammate = teammate,
                        )
                    )
                }
            }
        }
        return actions
    }

    override fun matchesClue (action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.gameAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val chop = getChop(receiverHand)
        val focus = getFocusedSlot(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        if (chop.index != focus.index) {
            return false
        }
        if (clueReceiver == playerPOV.playerId) {
            val ownHand = playerPOV.ownHand
            val focusedSlot = ownHand.getSlot(focus.index)
            return focusedSlot.getPossibleIdentities()
                .any { playerPOV.globallyAvailableInfo.isCritical(it) }
        } else {
            val teammate = playerPOV.getTeammate(clueReceiver)
            val focusedCard = teammate.getCardAtSlot(focus.index)
            return playerPOV.globallyAvailableInfo.isCritical(focusedCard)
        }
    }
}
