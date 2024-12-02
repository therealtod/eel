package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class RankClueAction(
    override val target: Int,
    val value: Int,
) : Action(HanabLiveActionType.RANK_CLUE, target)
