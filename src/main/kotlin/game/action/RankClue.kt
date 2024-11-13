package eelst.ilike.game.action

import eelst.ilike.game.entity.Rank
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

data class RankClue(
    val rank: Rank,
    override val receiver: PlayerId,
): Clue(
    value = rank,
    receiver = receiver,
) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getRanksTouching(card.rank).contains(rank)
    }
}
