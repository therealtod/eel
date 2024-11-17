package eelst.ilike.utils.model

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.action.Clue


@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class SlotDTO(
    val cardAbbreviation: String  = "x",
    val impliedIdentities: List<String> = emptyList(),
    val positiveClues : List<String> = emptyList(),
    val negativeClues : List<String> = emptyList(),
)
