package eelst.ilike.game

import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.Variant
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.suit.SuitId

data class NonPlayerRelatedGameData(
    val playingStacks: Map<SuitId, PlayingStack>,
    val trashPile: TrashPile,
    val strikes: Int,
    val clueTokens: Int,
    val players: Map<PlayerId, Player>
) {
    fun getCardsOnStacks(): List<HanabiCard> {
        return playingStacks.flatMap { it.value.cards }
    }

    /**
     * @return the updated [NonPlayerRelatedGameData] which result from a player playing the given [card]
     */
    fun getAfterPlaying(card: HanabiCard, variant: Variant): NonPlayerRelatedGameData {
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
            )
        } else {
            return this.copy(
                trashPile = trashPile.withAddedCard(card),
                strikes = strikes + 1,
            )
        }
    }

    /**
     * @return the updated [NonPlayerRelatedGameData] which result from a player discarding the given [card]
     */
    fun getAfterDiscarding(card: HanabiCard): NonPlayerRelatedGameData {
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
     * @return the updated [NonPlayerRelatedGameData] which results from a player giving a clue
     *
     * @throws [IllegalAccessException] if there are no clue tokens in the bank
     */
    fun getAfterPlayerCluing(): NonPlayerRelatedGameData {
        if (clueTokens < 1) {
            throw IllegalAccessException("A clue cannot be given if there are no clue tokens in the bank")
        }
        return this.copy(
            clueTokens = clueTokens - 1
        )
    }

    /**
     * @return the correct [PlayingStack] for the given [card]
     */
    fun getPlayingStackByCard(card: HanabiCard): PlayingStack {
        return playingStacks[card.suit.id]
            ?: throw IllegalStateException("The given $card does not correspond to any suit in this game")
    }

    val score: Int
        get() = getCardsOnStacks().size
}
