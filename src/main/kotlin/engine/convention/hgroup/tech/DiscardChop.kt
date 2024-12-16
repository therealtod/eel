package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.tech.DiscardTech
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DiscardChop : HGroupTech(), DiscardTech {
    override val name = "Discard Chop"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        return getChop(engineHandlerPlayer.hand, activePlayer).index == slot.index
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<DiscardAction> {
        return if (activePlayer.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                DiscardAction(
                    playerId = activePlayer.getOwnPlayerId(),
                    slotIndex = getChop(activePlayer.getOwnHand(), activePlayer).index
                )
            )
        } else emptySet()
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        TODO()
    }

    override fun matchesDiscard(action: ObservedDiscard, activePlayer: ActivePlayer): Boolean {
        TODO("Not yet implemented")
    }

    override fun getGeneratedKnowledge(action: ObservedAction, activePlayer: ActivePlayer): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }
}
