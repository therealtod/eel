package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.PlayTech
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.engine.player.PlayerPOV

object PlayKnownPlayable : HGroupTech(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
), PlayTech {
    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayAction> {
        return playerPOV
            .getOwnKnownPlayableSlots()
            .map {
                PlayAction(
                    playerId = playerPOV.playerId,
                    slotIndex = it.index
                )
            }.toSet()
    }
}
