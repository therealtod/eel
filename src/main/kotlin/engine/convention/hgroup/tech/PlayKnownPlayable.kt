package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.PlayTech
import eelst.ilike.game.action.Play

object PlayKnownPlayable : HGroupTech(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
), PlayTech {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        return playerPOV
            .getOwnKnownPlayableSlots()
            .map {
                ConventionalAction(
                    action = Play(it),
                    tech = PlayKnownPlayable
                )
            }.toSet()
    }
}
