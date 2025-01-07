package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.TableId


data class GameActionListData(
    val tableID: TableId,
    val list: List<HanabLiveGameActionData>
)

