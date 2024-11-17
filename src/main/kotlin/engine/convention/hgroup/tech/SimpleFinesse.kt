package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.hasCardOnFinessePosition
import eelst.ilike.game.entity.suite.Blue
import eelst.ilike.game.entity.suite.Green
import eelst.ilike.game.entity.suite.Purple
import eelst.ilike.game.entity.suite.Red
import eelst.ilike.game.entity.suite.Yellow

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
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