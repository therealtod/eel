package eelst.ilike.hanablive.model.dto.instruction

import com.fasterxml.jackson.annotation.JsonSubTypes
import eelst.ilike.hanablive.model.dto.command.GameActionType

@JsonSubTypes
sealed class GameActionData(
    val gameActionType: GameActionType
)
