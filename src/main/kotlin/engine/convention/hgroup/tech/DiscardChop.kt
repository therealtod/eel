package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.Discard
import eelst.ilike.engine.action.PlayerAction
import eelst.ilike.engine.convention.DiscardTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.player.PlayerPOV

object DiscardChop : HGroupTech(
    name = "Discard Chop",
    takesPrecedenceOver = emptySet(),
), DiscardTech {
    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayerAction> {
        return if (playerPOV.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                Discard(
                    from = playerPOV.playerId,
                    slotIndex = HGroupCommon.getChop(playerPOV.ownHand).index
                )
            )
        } else emptySet()
    }
}
