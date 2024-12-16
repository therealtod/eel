package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.convention.tech.PlayTech
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
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

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        val teammateKnowsOwnSlot = engineHandlerPlayer.getHandFromPlayerPOV().getSlot(slot.index) is KnownSlot
        return teammateKnowsOwnSlot &&
                slot.matches { _, card -> activePlayer.globallyAvailableInfo.isImmediatelyPlayable(card) }
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<PlayAction> {
        return activePlayer
            .getOwnHand()
            .filterIsInstance<KnownSlot>()
            .filter {
                activePlayer.globallyAvailableInfo.isImmediatelyPlayable(it.knownIdentity)
            }
            .map {
                PlayAction(
                    playerId = activePlayer.getOwnPlayerId(),
                    slotIndex = it.index
                )
            }.toSet()
    }

    override fun matchesPlay(action: ObservedPlay, activePlayer: ActivePlayer): Boolean {
        TODO("Not yet implemented")
    }

    override fun getGeneratedKnowledge(action: ObservedAction, activePlayer: ActivePlayer): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }
}
