package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.action.RankClue
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*

object FiveSave
    : SaveClue(
    name = "5-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.ownHand)
            val card = teammate.getCardAtSlot(chop.index)
            if (card.rank == Rank.FIVE) {
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
