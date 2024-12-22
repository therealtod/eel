package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.TableId

data class GameActionListData(
    val tableID: TableId,
    val list: List<HanabLiveGameActionData>
)

