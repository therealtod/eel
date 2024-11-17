package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

sealed class Action(
    open val type: HanabLiveActionType,
    open val target: Int,
)
