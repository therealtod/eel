package common.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming


@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class SlotCluesDTO(
    val positiveClues: List<String> = emptyList(),
    val negativeClues: List<String> = emptyList(),
)
