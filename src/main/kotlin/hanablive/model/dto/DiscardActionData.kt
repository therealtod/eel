package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class DiscardActionData(override val target: Int) : Action(HanabLiveActionType.DISCARD, target)
