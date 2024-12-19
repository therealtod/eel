package common.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import common.model.dto.PlayerDTO
import eelst.ilike.game.PlayerId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class PlayerPOVDTO(
    val playerId: PlayerId,
    val players: List<PlayerDTO>
)
