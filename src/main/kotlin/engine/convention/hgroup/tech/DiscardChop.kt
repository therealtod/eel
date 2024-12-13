package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.tech.DiscardTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DiscardChop : HGroupTech(), DiscardTech {
    override val name = "Discard Chop"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: VisibleTeammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        return getChop(teammate.hand, playerPOV).index == slotIndex
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<DiscardAction> {
        return if (playerPOV.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                DiscardAction(
                    playerId = playerPOV.getOwnPlayerId(),
                    slotIndex = getChop(playerPOV.getHand(playerPOV.getOwnPlayerId()), playerPOV).index
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
