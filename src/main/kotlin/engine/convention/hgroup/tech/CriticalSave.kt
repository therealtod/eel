package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayerAction> {
        val actions = mutableSetOf<PlayerAction>()

        playerPOV.forEachTeammate { teammate ->
            if (hasChop(teammate.hand)) {
                val chop = getChop(teammate.hand)
                val card = teammate.getSlot(chop.index).card
                if (appliesTo.contains(card.suite) &&
                    card.rank != Rank.FIVE &&
                    playerPOV.globallyAvailableInfo.isCritical(card) &&
                    !isGloballyKnownPlayable(card, playerPOV)
                ) {
                    actions.addAll(
                        getAllFocusingClues(
                            playerId = playerPOV.playerId,
                            slot = teammate.getSlot(chop.index),
                            teammate = teammate,
                        )
                    )
                }
            }
        }
        return actions
    }

    override fun matches(observedAction: ObservedAction, playerPOV: PlayerPOV): Boolean {
        val action = observedAction.action as GiveClue
        val receiver = action.to
        val previousTurnPOV = playerPOV.getPreviousTurnPOV()
        if(receiver != playerPOV.playerId) {
            val teammateSnapshot = previousTurnPOV.getTeammate(receiver)
            val focusIndex = HGroupCommon.getClueFocusSlotIndex(clue = action.clue, hand = teammateSnapshot.hand)
            val card = teammateSnapshot.getCardAtSlot(focusIndex)
            return playerPOV.globallyAvailableInfo.isCritical(card)
        } else {
            return playerPOV.getOwnSlot(focusIndex).getPossibleIdentities()
                .any { playerPOV.globallyAvailableInfo.isCritical(it) }
        }
    }
}
