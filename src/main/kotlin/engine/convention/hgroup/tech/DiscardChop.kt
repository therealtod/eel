package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.Discard
import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.DiscardTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.player.PlayerPOV

object DiscardChop : HGroupTech(
    name = "Discard Chop",
    takesPrecedenceOver = emptySet(),
), DiscardTech {
    override fun getGameActions(playerPOV: PlayerPOV): Set<GameAction> {
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
