package eelst.ilike.hanablive.entity.parsed

import eelst.ilike.hanablive.entity.dto.instruction.GameDrawActionData
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameActionData

class ParsedGameActionList(
    val initialDrawActions: List<GameDrawActionData>,
    val actionsByTurn: List<List<HanabLiveGameActionData>>,
) {
    val initialDrawActionsGroupedByPlayerIndex = initialDrawActions
        .groupBy { it.playerIndex }
}
