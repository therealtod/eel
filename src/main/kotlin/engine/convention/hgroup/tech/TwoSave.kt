package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.getChop
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow

object TwoSave : SaveClue(
    name = "2-Save",
    appliesTo = setOf(NoVarRed, NoVarYellow, NoVarGreen, NoVarBlue, NoVarPurple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            val chop = getChop(teammate.hand)
            val card = chop.getCard()
            if (card.rank == Rank.TWO
                && canBeTwoSaved(
                    card = card,
                    teammate = teammate,
                    playerPOV = playerPOV,
                )
            ) {
                actions.add(
                    ConventionalAction(
                        action = RankClue(Rank.TWO, receiver = teammate.playerId),
                        tech = this,
                    )
                )
            }
        }
        return actions.toSet()
    }

    private fun canBeTwoSaved(
        card: HanabiCard,
        teammate: Teammate,
        playerPOV: PlayerPOV,
    ): Boolean {
        return playerPOV.teammates.none { otherTeammate ->
            otherTeammate.playerId != teammate.playerId &&
                    otherTeammate.hand.copiesOf(card) == 1 &&
                    getChop(otherTeammate.hand).getCard() != card

        }
    }
}