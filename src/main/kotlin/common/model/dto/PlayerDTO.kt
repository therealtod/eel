package common.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.PlayerId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class PlayerDTO(
    val playerId: PlayerId,
    val hand: List<SlotDTO>,
)
