package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.player.ActivePlayer

interface PlayTech : ConventionTech {
    fun matchesPlay(action: ObservedPlay, activePlayer: ActivePlayer): Boolean

    override fun matches(action: ObservedAction, activePlayer: ActivePlayer): Boolean {
        return if (action is ObservedPlay) {
            matchesPlay(action, activePlayer)
        } else {
            false
        }
    }
}
