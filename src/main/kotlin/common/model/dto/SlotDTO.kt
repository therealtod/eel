package eelst.ilike.utils.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class SlotDTO(
    val card: String = "x",
    val positiveClues: List<String> = emptyList(),
    val negativeClues: List<String> = emptyList(),
    val thinks: String = "x"
)
