package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.PlayTech
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.action.Play

object PlayKnownPlayable : HGroupTech(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
), PlayTech {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        return playerPOV
            .getOwnKnownPlayableSlots()
            .map {
                ConventionalAction(
                    action = Play(it.index),
                    tech = PlayKnownPlayable
                )
            }.toSet()
    }
}
