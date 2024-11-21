package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasCardOnFinessePosition
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.suite.*

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayerAction> {
        val actions = mutableListOf<PlayerAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.hand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
                    if (playerPOV.getTeammates().any { otherTeammate ->
                            otherTeammate.playsBefore(teammate) &&
                                    hasCardOnFinessePosition(
                                        card = card.suite.cardBefore(card),
                                        teammate = otherTeammate,
                                    )
                        }) {
                        actions.addAll(
                            getAllFocusingClues(
                                playerId = playerPOV.playerId,
                                slot = teammate.getSlot(slot.index),
                                teammate = teammate,
                            )
                        )
                    }
                }
            }
        }
        return actions.toSet()
    }
}