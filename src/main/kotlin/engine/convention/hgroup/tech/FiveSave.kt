package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import eelst.ilike.game.entity.suite.*

object FiveSave
    : SaveClue(
    name = "5-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<GameAction> {
        val actions = mutableListOf<GameAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.ownHand)
            val card = teammate.getCardAtSlot(chop.index)
            if (card.rank == Rank.FIVE) {
                actions.add(
                     GiveClue(clue = RankClue(Rank.FIVE), to = teammate.playerId),
                )
            }
        }
        return actions.toSet()
    }
    override fun getConventionalActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        TODO()
    }
}
