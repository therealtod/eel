package eelst.ilike.utils.model

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.SuiteId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class GloballyAvailableInfoDTO(
    val suites: List<SuiteId>,
    val playingStacks: List<List<String>>,
    val trashPile: List<String>,
    val strikes: Int,
    val efficiency: Float,
    val pace: Int,
    val score: Int,
    val variant: String = "No Variant",
    val players: List<PlayerGloballyAvailableInfoDTO>,
)
