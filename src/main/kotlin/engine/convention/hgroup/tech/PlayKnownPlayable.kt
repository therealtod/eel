package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.PlayAction

object PlayKnownPlayable : HGroupTech<PlayAction>(
    name = "Play Known Playable",
    takesPrecedenceOver = emptySet()
) {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val slot = teammate.hand.getSlot(slotIndex)
        val card = slot.card
        return teammate.knows(slotIndex) && playerPOV.globallyAvailableInfo.isImmediatelyPlayable(card)
    }

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

    override fun getGeneratedKnowledge(action: ObservedAction<PlayAction>, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
