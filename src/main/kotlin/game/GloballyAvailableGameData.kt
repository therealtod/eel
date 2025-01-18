package eelst.ilike.game

import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.SlotFactory
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.variant.Variant
import eelst.ilike.game.exception.IllegalGameActionException

data class GloballyAvailableGameData(
    val variant: Variant,
    val playingStacks: Map<SuitId, PlayingStack>,
    val trashPile: TrashPile,
    val strikes: Int,
    val clueTokens: Int,
    private val currentDeckSize: Int = variant.getSuits().flatMap { it.getAllSuitCards() }.size,
    private val players: List<Player>,
    private val indexOfPlayerOnTurn: Int = 0,
) {
    val numberOfPlayers = players.size
    val possibleMaxScore = variant.getMaxScore()

    /**
     * @return true if a certain card is already on the playing stacks
     */
    fun isAlreadyPlayed(card: HanabiCard): Boolean {
        return getPlayingStackByCard(card).contains(card)
    }


    /**
     * @return true if the given [card] is the only copy of [HanabiCard] of its kind which is both not trash and not in
     * the trash pile
     *
     * TODO: In TIIAH variants this is actually not globally available info
     */
    fun isCritical(
        card: HanabiCard,
    ): Boolean {
        return !isAlreadyPlayed(card) && trashPile.copiesOf(card) == card.suit.copiesOf(card.rank) - 1
    }

    /**
     * @return all the cards currently played on the playing stacks
     */
    fun getCardsOnStacks(): List<HanabiCard> {
        return playingStacks.flatMap { it.value.cards }
    }

    fun getAfterDraw(drawAction: DrawAction): GloballyAvailableGameData {
        val playerIndex = drawAction.actionExecutor.playerIndex
        val newSlot = SlotFactory.createEmptySlot()
        val updatedPlayer = players[playerIndex].getUpdatedAfterDrawing(slot = newSlot)
        val updatedPlayers = players.toMutableList()
        updatedPlayers[playerIndex] = updatedPlayer
        return this.copy(
            currentDeckSize = currentDeckSize - 1,
            players = updatedPlayers,
        )
    }

    /**
     * @return the updated [GloballyAvailableGameData] which result from a player playing a non specified card
     */
    fun getAfterPlay(playAction: PlayAction): GloballyAvailableGameData {
        val playerIndex = playAction.actionExecutor.playerIndex
        val updatedPlayer = players[playerIndex]
            .getUpdatedAfterPlaying(playAction.slotIndex)
        val updatedPlayers = players.toMutableList()
        updatedPlayers[playerIndex] = updatedPlayer
        return this.copy(
            indexOfPlayerOnTurn = (indexOfPlayerOnTurn + 1) % numberOfPlayers,
            players = updatedPlayers,
        )
    }

    /**
     * @return the updated [GloballyAvailableGameData] which result from a player playing the given [card]
     */
    fun getAfterPlaying(playAction: PlayAction, card: HanabiCard): GloballyAvailableGameData {
        val playerIndex = playAction.actionExecutor.playerIndex
        val stack = getPlayingStackByCard(card)
        val updatedStack = stack.getAfterPlaying(card, variant)
        val updatedPlayer = players[playerIndex]
            .getUpdatedAfterPlaying(playAction.slotIndex)
        val updatedPlayers = players.toMutableList()
        updatedPlayers[playerIndex] = updatedPlayer
        val isPlayedSuccessfully = updatedStack.contains(card)
        if (isPlayedSuccessfully) {
            val newStacks = playingStacks
                .minus(stack.suit.id)
                .plus(Pair(updatedStack.suit.id, updatedStack))
            val newClueTokens = if (clueTokens < GameConstants.MAX_CLUE_TOKENS_COUNT && updatedStack.isComplete())
                clueTokens + 1
            else
                clueTokens
            return this.copy(
                playingStacks = newStacks,
                clueTokens = newClueTokens,
                indexOfPlayerOnTurn = (indexOfPlayerOnTurn + 1) % numberOfPlayers,
                players = updatedPlayers,
            )
        } else {
            return this.copy(
                trashPile = trashPile.withAddedCard(card),
                strikes = strikes + 1,
                indexOfPlayerOnTurn = (indexOfPlayerOnTurn + 1) % numberOfPlayers,
                players = updatedPlayers,
            )
        }
    }

    /**
     * @return the updated [GloballyAvailableGameData] which result from a player discarding a non specified card
     *
     * @throws [IllegalGameActionException] when discarding is not a legal action
     */
    fun getAfterDiscard(discardAction: DiscardAction): GloballyAvailableGameData {
        val playerIndex = discardAction.actionExecutor.playerIndex
        val updatedPlayer = players[playerIndex]
            .getUpdatedAfterDiscarding(discardAction.slotIndex)
        val updatedPlayers = players.toMutableList()
        updatedPlayers[playerIndex] = updatedPlayer
        return this.copy(
            indexOfPlayerOnTurn = (indexOfPlayerOnTurn + 1) % numberOfPlayers,
            players = updatedPlayers,
        )
    }

    /**
     * @return the updated [GloballyAvailableGameData] which result from a player discarding the given [card]
     *
     * @throws [IllegalGameActionException] when discarding is not a legal action
     */
    fun getAfterDiscarding(
        discardAction: DiscardAction,
        card: HanabiCard
    ): GloballyAvailableGameData {
        val playerIndex = discardAction.actionExecutor.playerIndex
        val updatedPlayer = players[playerIndex].getUpdatedAfterDiscarding(discardAction.slotIndex)
        val updatedPlayers = players.toMutableList()
        updatedPlayers[playerIndex] = updatedPlayer
        if (clueTokens == GameConstants.MAX_CLUE_TOKENS_COUNT) {
            throw IllegalGameActionException("It's not allowed to discard when the team has the maximum amount of clues in the bank")
        }
        val newTrashPile = trashPile.withAddedCard(card)
        val newClueTokens = if (clueTokens < GameConstants.MAX_CLUE_TOKENS_COUNT)
            clueTokens + 1
        else
            clueTokens
        return this.copy(
            trashPile = newTrashPile,
            clueTokens = newClueTokens,
            indexOfPlayerOnTurn = (indexOfPlayerOnTurn + 1) % numberOfPlayers,
            players = updatedPlayers,
        )
    }

    /**
     * @return the updated [GloballyAvailableGameData] which results from a player giving a clue
     *
     * @throws [IllegalGameActionException] if there are no clue tokens in the bank
     */
    fun getAfterClueGiven(
        clueAction: ClueAction,
        touchedSlotIndexes: Collection<Int>
    ): GloballyAvailableGameData {
        val playerIndex = clueAction.clueReceiver.playerIndex
        val updatedPlayer = players[playerIndex].getUpdatedAfterClueGiven(clueAction.value, touchedSlotIndexes)
        val updatedPlayers = players.toMutableList()
        updatedPlayers[playerIndex] = updatedPlayer
        if (clueTokens < 1) {
            throw IllegalGameActionException("A clue cannot be given if there are no clue tokens in the bank")
        }
        return this.copy(
            clueTokens = clueTokens - 1,
            indexOfPlayerOnTurn = (indexOfPlayerOnTurn + 1) % numberOfPlayers,
            players = updatedPlayers,
        )
    }

    /**
     * @return how many card are left in the deck
     */
    fun getCurrentDeckSize(): Int {
        return currentDeckSize
    }

    /**
     * @return the correct [PlayingStack] for the given [card]
     */
    fun getPlayingStackByCard(card: HanabiCard): PlayingStack {
        return playingStacks[card.suit.id]
            ?: throw IllegalStateException("The given $card does not correspond to any suit in this game")
    }

    /**
     * @return n as in "the slot is n-from playable"
     */
    fun getAwayValue(card: HanabiCard): Int {
        val stack = getPlayingStackByCard(card)
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
    fun isImmediatelyPlayable(card: HanabiCard): Boolean {
        return getAwayValue(card) == 0
    }

    /**
     * @return the [PlayerMetadata] of the player on turn
     */
    fun getPlayerOnTurn(): Player {
        return players[indexOfPlayerOnTurn]
    }

    fun getGloballyVisibleCards(): Collection<HanabiCard> {
        return globallyVisibleCards
    }

    /**
     * @return the [Player] with the given [playerIndex]
     */
    fun getPlayer(playerIndex: Int): Player {
        return players[playerIndex]
    }

    /**
     * @return the [Player]s participating in the game
     */
    fun getPlayers(): List<Player> {
        return players
    }

    /**
     * Current score
     */
    val score = getCardsOnStacks().size

    /**
     * The maximum score achievable at the beginning of the game
     */
    val initialMaxScore = playingStacks.values.fold(0) { acc, playingStack ->
        acc + playingStack.maxSize
    }

    private val globallyVisibleCards = playingStacks.values.flatMap { it.cards } + trashPile.cards

    /**
     * All the cards contained in deck at the very start of the game
     */
    private val cardsInInitialDeck = variant.getSuits().flatMap { it.getAllSuitCards() }

    /**
     * See https://github.com/Hanabi-Live/hanabi-live/blob/main/docs/features.md#pace
     */
    val pace = score + cardsInInitialDeck.size + numberOfPlayers - possibleMaxScore
}
