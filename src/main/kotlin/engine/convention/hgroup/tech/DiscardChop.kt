package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.action.DiscardAction

object DiscardChop : HGroupTech<DiscardAction>(
    name = "Discard Chop",
    takesPrecedenceOver = emptySet(),
) {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        return HGroupCommon.getChop(teammate.hand).index == slotIndex
    }

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
