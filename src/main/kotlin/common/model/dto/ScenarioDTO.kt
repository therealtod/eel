package eelst.ilike.utils.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import common.model.dto.GloballyAvailableInfoDTO
import common.model.dto.PlayerPOVDTO


@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class ScenarioDTO(
    val id: Int,
    val description: String,
    val globallyAvailableInfo: GloballyAvailableInfoDTO,
    val playerPOV: PlayerPOVDTO,
)
