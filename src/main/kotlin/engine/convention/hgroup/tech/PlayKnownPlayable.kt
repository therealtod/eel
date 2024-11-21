package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.engine.action.Play
import eelst.ilike.engine.convention.PlayTech
import eelst.ilike.engine.player.PlayerPOV

object PlayKnownPlayable : HGroupTech(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
), PlayTech {
    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayerAction> {
        return playerPOV
            .getOwnKnownPlayableSlots()
            .map {
                Play(
                    from = playerPOV.playerId,
                    slotIndex = it.index,
                )
            }.toSet()
    }
}
