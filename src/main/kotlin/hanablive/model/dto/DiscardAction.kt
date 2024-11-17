package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class DiscardAction(override val target: Int, ): Action(HanabLiveActionType.DISCARD, target)
