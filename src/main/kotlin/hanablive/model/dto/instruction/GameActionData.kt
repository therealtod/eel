package eelst.ilike.hanablive.model.dto.instruction

import com.fasterxml.jackson.annotation.JsonSubTypes
import com.fasterxml.jackson.annotation.JsonTypeInfo
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.hanablive.HanabLiveGame
import eelst.ilike.hanablive.model.dto.command.GameActionType

@JsonTypeInfo(
    use = JsonTypeInfo.Id.NAME,
    include = JsonTypeInfo.As.EXISTING_PROPERTY,
    property = "type", visible = true
)
@JsonSubTypes(
    JsonSubTypes.Type(value = GameDrawActionData::class, name = "draw"),
    JsonSubTypes.Type(value = GameDiscardActionData::class, name = "discard"),
    JsonSubTypes.Type(value = GameClueActionData::class, name = "clue"),
    JsonSubTypes.Type(value = GameTurnActionData::class, name = "turn"),
    JsonSubTypes.Type(value = GameStatusActionData::class, name = "status"),
)
sealed class GameActionData(
    val type: GameActionType
) {
    abstract fun toStandardFormatAction(game: HanabLiveGame): GameAction
}
