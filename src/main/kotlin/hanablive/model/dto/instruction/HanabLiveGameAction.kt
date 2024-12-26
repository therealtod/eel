package eelst.ilike.hanablive.model.dto.instruction

import eelst.ilike.hanablive.model.TableId


data class HanabLiveGameAction(
    val tableID: TableId,
    val action: HanabLiveGameActionData
)
