package eelst.ilike.game

import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.Common

data class GloballyAvailableInfo(
    val playingStacks: Map<SuiteId, PlayingStack>,
    val suites: Set<Suite>,
    val trashPile: TrashPile,
    val strikes: Int,
    val efficiency: Float,
    val pace: Int,
    val score: Int,
    val variant: Variant,
    val players: Map<PlayerId, GloballyAvailablePlayerInfo>,
) {
    val cardsOnStacks = playingStacks.flatMap { it.value.cards }
    val handsSize = Common.getHandSize(players.size)

    fun getStackForCard(card: HanabiCard): PlayingStack {
        val suiteId = card.suite.id
        return playingStacks[suiteId]
            ?: throw IllegalArgumentException("No stack in this game corresponding to the card $card")
    }

    fun isAlreadyPlayed(card: HanabiCard): Boolean {
        return getStackForCard(card).contains(card)
    }

    fun isCritical(
        card: HanabiCard,
    ): Boolean {
        return !isAlreadyPlayed(card) && trashPile.copiesOf(card) == card.suite.copiesOf(card.rank) - 1
    }

    /**
     * @return n as in "the slot is n-from playable"
     */
    fun getGlobalAwayValue(card: HanabiCard): Int {
        val stack = getStackForCard(card)
        val suite = card.suite
        return if (stack.isEmpty()){
            suite.getPlayingOrder(card) - 1
        } else {
            suite.getPlayingOrder(card) - suite.getPlayingOrder(stack.currentCard()) - 1
        }
    }

    fun isImmediatelyPlayable(card: HanabiCard): Boolean {
        return getGlobalAwayValue(card) == 0
    }

    fun getPlayerInfo(playerId: PlayerId): GloballyAvailablePlayerInfo {
        return players[playerId]
            ?: throw IllegalArgumentException("No player with id: $playerId in this game")
    }
}