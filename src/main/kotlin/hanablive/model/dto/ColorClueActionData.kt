package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class ColorClueActionData(
    override val target: Int,
    val value: Int,
) : Action(HanabLiveActionType.COLOR_CLUE, target)
