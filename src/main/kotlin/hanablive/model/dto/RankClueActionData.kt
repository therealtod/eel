package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class RankClueActionData(
    override val target: Int,
    val value: Int,
) : Action(HanabLiveActionType.RANK_CLUE, target)
