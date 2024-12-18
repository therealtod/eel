package eelst.ilike.hanablive.model.dto.instruction

import com.fasterxml.jackson.annotation.JsonSubTypes
import com.fasterxml.jackson.annotation.JsonTypeInfo
import eelst.ilike.hanablive.model.dto.command.GameActionType

@JsonTypeInfo(
    use = JsonTypeInfo.Id.NAME,
    include = JsonTypeInfo.As.EXISTING_PROPERTY,
    property = "type", visible = true
)
@JsonSubTypes(
    JsonSubTypes.Type(value = GameDrawActionData::class, name = "draw")
)
sealed class GameActionData(
    val type: GameActionType
)
