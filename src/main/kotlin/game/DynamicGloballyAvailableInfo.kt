package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.suite.SuiteId

data class DynamicGloballyAvailableInfo(
    val playingStacks: Map<SuiteId, PlayingStack>,
    val trashPile: TrashPile,
    val strikes: Int,
    val clueTokens: Int,
    val pace: Int,
    val efficiency: Float,
)
