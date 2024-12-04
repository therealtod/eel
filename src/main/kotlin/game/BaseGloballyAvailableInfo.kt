package eelst.ilike.game

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.utils.Utils

abstract class BaseGloballyAvailableInfo(
    playersIds: Set<PlayerId>,
    globallyAvailablePlayerInfo: Set<GloballyAvailablePlayerInfo>,
    private val dynamicGloballyAvailableInfo: DynamicGloballyAvailableInfo,
) : GloballyAvailableInfo {
    override val playingStacks = dynamicGloballyAvailableInfo.playingStacks

    override val trashPile = dynamicGloballyAvailableInfo.trashPile

    override val strikes = dynamicGloballyAvailableInfo.strikes

    override val clueTokens = dynamicGloballyAvailableInfo.clueTokens

    override val score = getCardsOnStacks().size

    override val pace = dynamicGloballyAvailableInfo.pace

    override val efficiency = dynamicGloballyAvailableInfo.efficiency

    override val players = playersIds.zip(globallyAvailablePlayerInfo).associate { it.first to it.second }

    override val defaultHandsSize = Utils.getHandSize(playersIds.size)

    override fun getCardsOnStacks(): List<HanabiCard> {
        return dynamicGloballyAvailableInfo.playingStacks.values.flatten()
    }

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

    override fun getCluableRanks(): Set<Rank> {
        return variant.getCluableRanks()
    }

    override fun getCluableColors(): Set<Color> {
        return variant.getCluableColors()
    }

    override fun getPlayerInfo(playerIndex: Int): GloballyAvailablePlayerInfo {
        return players.values.find { it.playerIndex == playerIndex }
            ?: throw IllegalArgumentException("Could not find any player with player index $playerIndex")
    }
}
