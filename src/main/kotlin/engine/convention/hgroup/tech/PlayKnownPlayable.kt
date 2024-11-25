package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.engine.player.PlayerPOV

object PlayKnownPlayable : HGroupTech<PlayAction>(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
) {
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
