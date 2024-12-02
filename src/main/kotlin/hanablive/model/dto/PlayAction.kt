package eelst.ilike.hanablive.model.dto

import eelst.ilike.hanablive.HanabLiveActionType

data class PlayAction(override val target: Int) : Action(HanabLiveActionType.PLAY, target)
