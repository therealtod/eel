package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.action.RankClue
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.PlayerPOVImpl
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*

object TwoSave : SaveClue(
    name = "2-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.forEachTeammate{ teammate ->
            val chop = getChop(teammate.ownHand)
            val card = teammate.getSlot(chop.index).card
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
        return playerPOV.getTeammates().none { otherTeammate ->
            otherTeammate.playerId != teammate.playerId &&
                    otherTeammate.ownHand.copiesOf(card) == 1 &&
                    otherTeammate.getCardAtSlot(getChop(otherTeammate.ownHand).index) != card
        }
    }
}