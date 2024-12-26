package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class EndGameAction(
    override val target: Int,
    val value: Int
) : Action(
    type = HanabLiveActionType.END_GAME,
    target = target,
)
