package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.player.PlayerPOV

interface PlayTech : ConventionTech {
    fun matchesPlay(action: ObservedPlay, playerPOV: PlayerPOV): Boolean

    override fun matches(action: ObservedAction, playerPOV: PlayerPOV): Boolean {
        return if (action is ObservedPlay) {
            matchesPlay(action, playerPOV)
        } else {
            false
        }
    }
}
