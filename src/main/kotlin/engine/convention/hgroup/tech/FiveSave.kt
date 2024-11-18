package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*

object FiveSave
    : SaveClue(
    name = "5-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            val chop = getChop(teammate.hand)
            val card = teammate.getSlot(chop.index).card
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
