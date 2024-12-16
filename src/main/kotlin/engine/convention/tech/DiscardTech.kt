package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.player.ActivePlayer

interface DiscardTech : ConventionTech {
    fun matchesDiscard(action: ObservedDiscard, activePlayer: ActivePlayer): Boolean

    override fun matches(action: ObservedAction, activePlayer: ActivePlayer): Boolean {
        return if (action is ObservedDiscard) {
            matchesDiscard(action, activePlayer)
        } else {
            false
        }
    }
}
