package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasCardOnFinessePosition
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.entity.suite.*

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
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
                            getAllFocusingActions(
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
}