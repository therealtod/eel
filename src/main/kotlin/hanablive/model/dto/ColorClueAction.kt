package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class ColorClueAction(
    override val target: Int,
    val value: Int,
) : Action(HanabLiveActionType.COLOR_CLUE, target)
