package common.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.entity.suite.SuiteId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class GloballyAvailableInfoDTO(
    val suits: List<SuiteId>,
    val playingStacks: List<List<String>>,
    val trashPile: List<String>,
    val strikes: Int,
    val efficiency: Float,
    val pace: Int,
    val variant: String = "No Variant",
    val clueTokens: Int,
)
