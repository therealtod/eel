package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.getChop
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Blue
import eelst.ilike.game.entity.suite.Green
import eelst.ilike.game.entity.suite.Purple
import eelst.ilike.game.entity.suite.Red
import eelst.ilike.game.entity.suite.Yellow

object TwoSave : SaveClue(
    name = "2-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            val chop = getChop(teammate.hand)
            val card = teammate.hand.getSlot(chop.index).card
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
                    otherTeammate.getCardAtSlot(getChop(otherTeammate.hand).index) != card
        }
    }
}