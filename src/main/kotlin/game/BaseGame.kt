package eelst.ilike.game

import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.suit.SuitId


abstract class BaseGame(
    override val variant: Variant,
    protected val nonPlayerRelatedGameData: NonPlayerRelatedGameData
) : Game {
    constructor(
        variant: Variant,
        playingStacks: Map<SuitId,PlayingStack>,
        trashPile: TrashPile,
        strikes: Int,
        clueTokens: Int,
        players: Map<PlayerId, Player>,
    ) : this(
        variant = variant,
        NonPlayerRelatedGameData(
            playingStacks = playingStacks,
            trashPile = trashPile,
            strikes = strikes,
            clueTokens = clueTokens,
            players = players,
        )
    )

    final override val numberOfPlayers = players.size

    final override val suits = variant.suits

    private val cardsInDeck = suits.flatMap { it.getAllCards() }.size
    private val maxScore = suits.size * 5
    protected val availableColors = suits.flatMap { it.getAssociatedColors() }
    protected val availableRanks = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)

    override val pace = score + cardsInDeck + numberOfPlayers - maxScore

    override val defaultHandsSize = GameUtils.getHandSize(numberOfPlayers)

    override fun getCardsOnStacks(): List<HanabiCard> {
        return nonPlayerRelatedGameData.getCardsOnStacks()
    }

    final override val score: Int
        get() = getCardsOnStacks().size

    override fun getStackForCard(card: HanabiCard): PlayingStack {
        val suitId = card.suit.id
        return playingStacks[suitId]
            ?: throw IllegalArgumentException("No stack in this game corresponding to the card $card")
    }

    override fun isAlreadyPlayed(card: HanabiCard): Boolean {
        return getStackForCard(card).contains(card)
    }

    /**
     * @return true if the given [card] is the only copy of [HanabiCard] of its kind left in the game
     * which is not in the trash
     */
    override fun isCritical(
        card: HanabiCard,
    ): Boolean {
        return !isAlreadyPlayed(card) && trashPile.copiesOf(card) == card.suit.copiesOf(card.rank) - 1
    }

    /**
     * @return n as in "the slot is n-from playable"
     */
    override fun getGlobalAwayValue(card: HanabiCard): Int {
        val stack = getStackForCard(card)
        val suit = card.suit
        return if (stack.isEmpty()) {
            suit.getPlayingOrder(card) - 1
        } else {
            suit.getPlayingOrder(card) - suit.getPlayingOrder(stack.currentCard()) - 1
        }
    }

    /**
     * @return true if the card can be immediately successfully played on the stacks
     */
    override fun isImmediatelyPlayable(card: HanabiCard): Boolean {
        return getGlobalAwayValue(card) == 0
    }

    /**
     * @return the [Player] playing in this game with the given [playerId]
     */
    override fun getPlayer(playerId: PlayerId): Player {
        return players[playerId]
            ?: throw IllegalArgumentException("No player with id: $playerId in this game")
    }

    /**
     * @return the [Player] playing in this game with the given [playerIndex]
     */
    override fun getPlayer(playerIndex: Int): Player {
        return players.values.find { it.playerIndex == playerIndex }
            ?: throw IllegalArgumentException("Could not find any player with player index $playerIndex")
    }

    override fun getAvailableClueValues(): Set<ClueValue> {
        return (availableColors + availableRanks).toSet()
    }

    override fun getAfter(drawAction: DrawAction): Game {
        val player = getPlayer(drawAction.playerId)
        val updatedPlayer = player.getAfterDrawing(drawAction.newSlot)
        val updatedPlayers = players
            .minus(updatedPlayer.playerId)
            .plus(Pair(updatedPlayer.playerId, updatedPlayer))
        return
    }
}
