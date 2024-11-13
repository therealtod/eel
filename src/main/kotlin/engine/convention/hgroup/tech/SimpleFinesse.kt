package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.hasCardOnFinessePosition
import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(NoVarRed, NoVarYellow, NoVarGreen, NoVarBlue, NoVarPurple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            teammate.hand.forEach { slot ->
                val card = slot.getCard()
                if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
                    if (playerPOV.teammates.any { otherTeammate ->
                            otherTeammate.playsBefore(teammate) &&
                                    hasCardOnFinessePosition(
                                        card = card.suite.cardBefore(card),
                                        teammate = otherTeammate,
                                        playerPOV = playerPOV
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