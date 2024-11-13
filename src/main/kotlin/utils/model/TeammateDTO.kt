package eelst.ilike.utils.model

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.PlayerId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class TeammateDTO(
    val playerId: PlayerId,
    val playerPOV: PlayerPOVDTO,
    val hand: List<SlotDTO>,
)