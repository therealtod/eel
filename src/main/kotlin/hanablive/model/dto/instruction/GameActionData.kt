package eelst.ilike.hanablive.model.dto.command

import com.fasterxml.jackson.annotation.JsonSubTypes

@JsonSubTypes
sealed class GameActionData(
    val gameActionType: GameActionType
)
