package eelst.ilike.game.entity.action

import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard

data class RankClue(val rank: Rank): Clue(rank) {
    override fun touches(card: HanabiCard): Boolean {
        return card.suite.getRanksTouching(card.rank).contains(rank)
    }
}