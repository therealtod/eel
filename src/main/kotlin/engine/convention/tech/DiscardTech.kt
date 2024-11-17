package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.player.PlayerPOV

abstract class DiscardTech : ConventionTech {
    abstract fun matchesDiscard(action: ObservedDiscard, playerPOV: PlayerPOV): Boolean

    override fun matches(action: ObservedAction, playerPOV: PlayerPOV): Boolean {
        return if (action is ObservedDiscard) {
            matchesDiscard(action, playerPOV)
        } else {
            false
        }
    }
}
