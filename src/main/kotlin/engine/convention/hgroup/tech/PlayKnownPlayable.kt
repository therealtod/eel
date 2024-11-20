package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.PlayTech
import eelst.ilike.engine.action.Play
import eelst.ilike.engine.player.PlayerPOV

object PlayKnownPlayable : HGroupTech(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
), PlayTech {
    override fun getGameActions(playerPOV: PlayerPOV): Set<GameAction> {
        return playerPOV
            .getOwnKnownPlayableSlots()
            .map {
                Play(it.index)
            }.toSet()
    }
    override fun getConventionalActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        TODO()
    }
}
