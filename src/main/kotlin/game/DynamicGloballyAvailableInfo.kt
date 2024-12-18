package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.SuiteId

data class DynamicGloballyAvailableInfo(
    val playingStacks: Map<SuiteId, PlayingStack>,
    val trashPile: TrashPile,
    val strikes: Int,
    val clueTokens: Int,
) {
    fun getCardsOnStacks(): List<HanabiCard> {
        return playingStacks.flatMap { it.value.cards }
    }

    val score: Int
        get() = getCardsOnStacks().size
}
