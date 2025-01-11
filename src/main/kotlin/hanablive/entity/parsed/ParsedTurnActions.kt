package eelst.ilike.hanablive.entity.parsed

import eelst.ilike.hanablive.entity.dto.instruction.GameStatusActionData
import eelst.ilike.hanablive.entity.dto.instruction.GameTurnActionData
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameAction

data class ParsedTurnActions(
    val playerAction: HanabLiveGameAction,
    val statusAction: GameStatusActionData,
    val turnAction: GameTurnActionData,
    val strikeHappened: Boolean,
)
