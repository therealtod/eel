package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow

object DirectPlayClue: PlayClue(
    name = "Direct Play Clue",
    appliesTo = setOf(NoVarRed, NoVarYellow, NoVarGreen, NoVarBlue, NoVarPurple),
    takesPrecedenceOver = emptySet(),
    ) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            teammate.hand.forEach { slot ->
                val card = slot.getCard()
                if (!teammate.knows(slot.index) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0) {
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
        return actions.toSet()
    }
}
