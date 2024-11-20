package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasCardOnFinessePosition
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.suite.*

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<GameAction> {
        val actions = mutableListOf<GameAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.hand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
                    if (playerPOV.teammates.any { otherTeammate ->
                            otherTeammate.playsBefore(teammate) &&
                                    hasCardOnFinessePosition(
                                        card = card.suite.cardBefore(card),
                                        teammate = otherTeammate,
                                    )
                        }) {
                        actions.addAll(
                            getAllFocusingClues(
                                card = card,
                                slot = slot,
                                teammate = teammate,
                            )
                        )
                    }
                }
            }
        }
        return actions.toSet()
    }
    override fun getConventionalActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        TODO()
    }
}