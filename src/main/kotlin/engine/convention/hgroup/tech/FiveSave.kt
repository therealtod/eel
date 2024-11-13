package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.getChop
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow

object FiveSave
    : SaveClue(
    name = "5-Save",
    appliesTo = setOf(NoVarRed, NoVarYellow, NoVarGreen, NoVarBlue, NoVarPurple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            val chop = getChop(teammate.hand)
            if (chop.getCard().rank == Rank.FIVE) {
                actions.add(
                    ConventionalAction(
                        action = RankClue(rank = Rank.FIVE, receiver = teammate.playerId),
                        tech = this
                    )
                )
            }
        }
        return actions.toSet()
    }
}
