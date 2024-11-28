package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.action.PlayAction

abstract class PlayTech: ConventionTech {
    abstract fun matchesPlay(action: ObservedPlay, playerPOV: PlayerPOV): Boolean

    override fun matches(action: ObservedAction, playerPOV: PlayerPOV): Boolean {
        return if (action is ObservedPlay) {
            matchesPlay(action, playerPOV)
        } else {
            false
        }
    }
}
