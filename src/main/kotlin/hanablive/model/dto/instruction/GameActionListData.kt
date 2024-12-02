package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameActionData

data class GameActionListData(
    val tableID: TableId,
    val list: List<GameActionData>
)

