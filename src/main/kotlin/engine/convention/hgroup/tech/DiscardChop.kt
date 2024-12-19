package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.tech.DiscardTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
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

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean {
        return getChop(teammate.hand, playerPOV).index == slot.index
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<DiscardAction> {
        return if (playerPOV.game.clueTokens < 8) {
            return setOf(
                DiscardAction(
                    playerId = playerPOV.getOwnPlayerId(),
                    slotIndex = getChop(playerPOV.getOwnHand(), playerPOV).index
                )
            )
        } else emptySet()
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        TODO()
    }

    override fun matchesDiscard(action: ObservedDiscard, playerPOV: PlayerPOV): Boolean {
        TODO("Not yet implemented")
    }

    override fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }
}
