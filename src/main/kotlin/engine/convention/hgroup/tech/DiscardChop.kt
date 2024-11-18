package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.DiscardTech
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.action.Discard

object DiscardChop : HGroupTech(
    name = "Discard Chop",
    takesPrecedenceOver = emptySet(),
), DiscardTech {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        return if (playerPOV.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                ConventionalAction(
                    action = Discard(HGroupCommon.getChop(playerPOV.ownHand).index),
                    tech = DiscardChop,
                )
            )
        } else emptySet()
    }
}
