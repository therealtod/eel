package eelst.ilike.utils.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.PlayerId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class PlayerGloballyAvailableInfoDTO(
    val playerId: PlayerId,
    val slotClues: List<SlotCluesDTO> = emptyList(),
)