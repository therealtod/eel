package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.convention.tech.PlayTech
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object PlayKnownPlayable : HGroupTech(), PlayTech {
    override val name = "Play Known Playable"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean {
        val teammateKnowsOwnSlot = teammate.getHandFromPlayerPOV().getSlot(slot.index) is KnownSlot
        return teammateKnowsOwnSlot &&
                slot.matches { _, card -> playerPOV.game.isImmediatelyPlayable(card) }
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<PlayAction> {
        return playerPOV
            .getOwnHand()
            .filterIsInstance<KnownSlot>()
            .filter {
                playerPOV.game.isImmediatelyPlayable(it.knownIdentity)
            }
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

    override fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }
}
