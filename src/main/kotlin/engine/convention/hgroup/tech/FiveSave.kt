package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.RankClueAction
import eelst.ilike.game.entity.suite.*

object FiveSave
    : SaveClue(
    name = "5-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.ownHand)
            val card = teammate.getCardAtSlot(chop.index)
            if (card.rank == Rank.FIVE) {
                actions.add(
                     RankClueAction(
                         clueGiver = playerPOV.playerId,
                         clueReceiver = teammate.playerId,
                         rank = Rank.FIVE,
                         ),
                )
            }
        }
        return actions.toSet()
    }
}
