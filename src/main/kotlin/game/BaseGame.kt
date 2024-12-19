package eelst.ilike.game

import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant

abstract class BaseGame(
    final override val variant: Variant,
    final override val playingStacks: Map<SuiteId, PlayingStack>,
    final override val trashPile: TrashPile,
    final override val strikes: Int,
    final override val clueTokens: Int,
    final override val players: Map<PlayerId, GloballyAvailablePlayerInfo>,
) : Game {
    constructor(
        variant: Variant,
        globallyAvailablePlayerInfo: Map<PlayerId, GloballyAvailablePlayerInfo>,
        dynamicGloballyAvailableInfo: DynamicGloballyAvailableInfo,
        ): this(
        variant = variant,
        playingStacks = dynamicGloballyAvailableInfo.playingStacks,
        trashPile = dynamicGloballyAvailableInfo.trashPile,
        strikes = dynamicGloballyAvailableInfo.strikes,
        clueTokens = dynamicGloballyAvailableInfo.clueTokens,
        players = globallyAvailablePlayerInfo
    )

    private val dynamicGloballyAvailableInfo = DynamicGloballyAvailableInfo(
        playingStacks = playingStacks,
        trashPile = trashPile,
        strikes = strikes,
        clueTokens = clueTokens,
    )

    final override val numberOfPlayers = players.size

    final override val suits = variant.suits

    private val cardsInDeck = suits.flatMap { it.getAllCards() }.size
    private val maxScore = suits.size * 5
    protected val availableColors = suits.flatMap { it.getAssociatedColors() }
    protected val availableRanks = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)

    override val pace = score + cardsInDeck + numberOfPlayers -maxScore

    override val defaultHandsSize = GameUtils.getHandSize(numberOfPlayers)
    override val efficiency: Float
        get() = 1.0F

    override fun getCardsOnStacks(): List<HanabiCard> {
        return dynamicGloballyAvailableInfo.getCardsOnStacks()
    }

    final override val score: Int
        get() = getCardsOnStacks().size

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

    override fun getPlayerInfo(playerIndex: Int): GloballyAvailablePlayerInfo {
        return players.values.find { it.playerIndex == playerIndex }
            ?: throw IllegalArgumentException("Could not find any player with player index $playerIndex")
    }

    override fun getAvailableClueValues(): Set<ClueValue> {
        return variant.getCluableRanks() +  variant.getCluableColors()
    }
}
