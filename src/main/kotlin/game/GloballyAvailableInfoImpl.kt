package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.Utils

data class GloballyAvailableInfoImpl(
    override val playingStacks: Map<SuiteId, PlayingStack>,
    override val suites: Set<Suite>,
    override val trashPile: TrashPile,
    override val strikes: Int,
    override val efficiency: Float,
    override val pace: Int,
    override val variant: Variant,
    override val clueTokens: Int,
    override val players: Map<PlayerId, GloballyAvailablePlayerInfo>,
) : GloballyAvailableInfo {
    override val cardsOnStacks = playingStacks.flatMap { it.value.cards }
    override val handsSize = Utils.getHandSize(players.size)
    override val numberOfPlayers = players.size
    override val score = cardsOnStacks.size

    override fun getStackForCard(card: HanabiCard): PlayingStack {
        val suiteId = card.suite.id
        return playingStacks[suiteId]
            ?: throw IllegalArgumentException("No stack in this game corresponding to the card $card")
    }

    override fun isAlreadyPlayed(card: HanabiCard): Boolean {
        return getStackForCard(card).contains(card)
    }

    override fun isCritical(
        card: HanabiCard,
    ): Boolean {
        return !isAlreadyPlayed(card) && trashPile.copiesOf(card) == card.suite.copiesOf(card.rank) - 1
    }

    /**
     * @return n as in "the slot is n-from playable"
     */
    override fun getGlobalAwayValue(card: HanabiCard): Int {
        val stack = getStackForCard(card)
        val suite = card.suite
        return if (stack.isEmpty()) {
            suite.getPlayingOrder(card) - 1
        } else {
            suite.getPlayingOrder(card) - suite.getPlayingOrder(stack.currentCard()) - 1
        }
    }

    override fun isImmediatelyPlayable(card: HanabiCard): Boolean {
        return getGlobalAwayValue(card) == 0
    }

    override fun getPlayerInfo(playerId: PlayerId): GloballyAvailablePlayerInfo {
        return players[playerId]
            ?: throw IllegalArgumentException("No player with id: $playerId in this game")
    }
}