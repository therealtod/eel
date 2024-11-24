package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.PlayerPOV

object DiscardChop : HGroupTech(
    name = "Discard Chop",
    takesPrecedenceOver = emptySet(),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<DiscardAction> {
        return if (playerPOV.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                DiscardAction(
                    playerId = playerPOV.playerId,
                    HGroupCommon.getChop(playerPOV.ownHand).index
                )
            )
        } else emptySet()
    }
}
