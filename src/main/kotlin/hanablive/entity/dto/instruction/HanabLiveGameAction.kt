package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.TableId


data class HanabLiveGameAction(
    val tableID: TableId,
    val action: HanabLiveGameActionData
)
