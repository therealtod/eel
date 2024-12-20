package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.SuiteId

data class DynamicGameData(
    val playingStacks: Map<SuiteId, PlayingStack>,
    val trashPile: TrashPile,
    val strikes: Int,
    val clueTokens: Int,
) {
    fun getCardsOnStacks(): List<HanabiCard> {
        return playingStacks.flatMap { it.value.cards }
    }

    fun withNewCardOnStacks(card: HanabiCard): DynamicGameData {
        val updatedPlayingStacks = playingStacks.mapValues {
            if(it.key == card.suite.id) {
                it.value.playCard(card)
            } else {
                it.value
            }
        }
        return DynamicGameData(
            playingStacks = updatedPlayingStacks,
            trashPile = trashPile,
            strikes = strikes,
            clueTokens = clueTokens,
        )
    }

    fun withNewStrike(card: HanabiCard): DynamicGameData {
        return DynamicGameData(
            playingStacks = playingStacks,
            trashPile = trashPile.withAddedCard(card),
            strikes = strikes + 1,
            clueTokens = clueTokens,
        )
    }

    fun withNewDiscard(card: HanabiCard): DynamicGameData {
        return DynamicGameData(
            playingStacks = playingStacks,
            trashPile = trashPile.withAddedCard(card),
            strikes = strikes,
            clueTokens = if(clueTokens < 8) clueTokens + 1 else clueTokens
        )
    }

    fun withAddedClueToken(): DynamicGameData {
        return DynamicGameData(
            playingStacks = playingStacks,
            trashPile = trashPile,
            strikes = strikes,
            clueTokens = if(clueTokens < 8) clueTokens + 1 else clueTokens
        )
    }

    fun withUsedClueToken(): DynamicGameData {
        return DynamicGameData(
            playingStacks = playingStacks,
            trashPile = trashPile,
            strikes = strikes,
            clueTokens = clueTokens - 1
        )
    }

    val score: Int
        get() = getCardsOnStacks().size
}
