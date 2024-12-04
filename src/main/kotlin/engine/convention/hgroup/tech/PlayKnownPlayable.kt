package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.convention.tech.PlayTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object PlayKnownPlayable : HGroupTech(), PlayTech {
    override val name = "Play Known Playable"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: VisibleTeammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardInSlot(slotIndex)
        return teammate.knowsIdentityOfOwnSlot(slotIndex) && playerPOV.globallyAvailableInfo.isImmediatelyPlayable(card)
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayAction> {
        return playerPOV
            .getOwnKnownPlayableSlots()
            .map {
                PlayAction(
                    playerId = playerPOV.getOwnPlayerId(),
                    slotIndex = it.index
                )
            }.toSet()
    }

    override fun matchesPlay(action: ObservedPlay, playerPOV: PlayerPOV): Boolean {
        TODO("Not yet implemented")
    }

    override fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
