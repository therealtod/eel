package eelst.ilike.game

import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.exception.IllegalGameActionException

data class GloballyAvailableGameData(
    val variant: Variant,
    val playingStacks: Map<SuitId, PlayingStack>,
    val trashPile: TrashPile,
    val strikes: Int,
    val clueTokens: Int,
    val numberOfPlayers: Int,
    val amountOfCardsPlayed: Int,
    val possibleMaxScore: Int,
) {
    /**
     * @return true if a certain card is already on the playing stacks
     */
    fun isAlreadyPlayed(card: HanabiCard): Boolean {
        return getPlayingStackByCard(card).contains(card)
    }


    /**
     * @return true if the given [card] is the only copy of [HanabiCard] of its kind which is both not trash and not in
     * the trash pile
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

    /**
     * @return the updated [GloballyAvailableGameData] which result from a player playing the given [card]
     */
    fun getAfterPlaying(card: HanabiCard, variant: Variant): GloballyAvailableGameData {
        val stack = getPlayingStackByCard(card)
        val updatedStack = stack.getAfterPlaying(card, variant)
        val isPlayedSuccessfully = updatedStack.contains(card)
        if (isPlayedSuccessfully) {
            val newStacks = playingStacks
                .minus(stack.suit.id)
                .plus(Pair(updatedStack.suit.id, updatedStack))
            val newClueTokens = if (clueTokens < GameConstants.MAX_CLUE_COUNT && updatedStack.isComplete())
                clueTokens + 1
            else
                clueTokens
            return this.copy(
                playingStacks = newStacks,
                clueTokens = newClueTokens,
                amountOfCardsPlayed = amountOfCardsPlayed + 1,
            )
        } else {
            return this.copy(
                trashPile = trashPile.withAddedCard(card),
                strikes = strikes + 1,
                amountOfCardsPlayed = amountOfCardsPlayed + 1,
            )
        }
    }

    /**
     * @return the updated [GloballyAvailableGameData] which result from a player discarding the given [card]
     *
     * @throws [IllegalGameActionException] when discarding is not a legal action
     */
    fun getAfterDiscarding(card: HanabiCard): GloballyAvailableGameData {
        if (clueTokens == GameConstants.MAX_CLUE_COUNT) {
            throw IllegalGameActionException("It's not allowed to discard when the team has the maximum amount of clues in the bank")
        }
        val newTrashPile = trashPile.withAddedCard(card)
        val newClueTokens = if(clueTokens < GameConstants.MAX_CLUE_COUNT)
            clueTokens + 1
        else
            clueTokens
        return this.copy(
            trashPile = newTrashPile,
            clueTokens = newClueTokens,
        )
    }

    /**
     * @return the updated [GloballyAvailableGameData] which results from a player giving a clue
     *
     * @throws [IllegalGameActionException] if there are no clue tokens in the bank
     */
    fun getAfterClueGiven(): GloballyAvailableGameData {
        if (clueTokens < 1) {
            throw IllegalGameActionException("A clue cannot be given if there are no clue tokens in the bank")
        }
        return this.copy(
            clueTokens = clueTokens - 1
        )
    }

    /**
     * @return how many card are left in the deck
     */
    fun getCurrentDeckSize(hands: Collection<Hand>): Int {
        val cardsInHands = hands.fold(0) { acc, hand -> acc + hand.size }

        return cardsInInitialDeck.size - (trashPile.cards.size + score + cardsInHands + amountOfCardsPlayed)
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
     * Current score
     */
    val score = getCardsOnStacks().size

    /**
     * The maximum score achievable at the beginning of the game
     */
    val initialMaxScore = playingStacks.values.fold(0) {
        acc, playingStack -> acc + playingStack.maxSize
    }

    /**
     * All the cards contained in deck at the very start of the game
     */
    private val cardsInInitialDeck = variant.suits.flatMap { it.getAllCards() }

    /**
     * See https://github.com/Hanabi-Live/hanabi-live/blob/main/docs/features.md#pace
     */
    val pace = score + cardsInInitialDeck.size + numberOfPlayers - possibleMaxScore
}
