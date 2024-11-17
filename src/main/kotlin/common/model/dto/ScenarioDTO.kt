package eelst.ilike.utils.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming


@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class ScenarioDTO(
    val id: Int,
    val description: String,
    val globallyAvailableInfo: GloballyAvailableInfoDTO,
    val playerPOV: PlayerPOVDTO,
)
