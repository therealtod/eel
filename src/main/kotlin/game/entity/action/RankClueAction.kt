package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard

class RankClueAction(
    clueGiver: PlayerId,
    clueReceiver: PlayerId,
    val rank: Rank
): ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = rank
) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getRanksTouching(card.rank).contains(rank)
    }
}
