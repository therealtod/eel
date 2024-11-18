package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.DiscardTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.action.Discard

object DiscardChop : HGroupTech(
    name = "Discard Chop",
    takesPrecedenceOver = emptySet(),
), DiscardTech {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        return if (playerPOV.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                ConventionalAction(
                    action = Discard(HGroupCommon.getChop(playerPOV.hand).index),
                    tech = DiscardChop,
                )
            )
        } else emptySet()
    }
}
