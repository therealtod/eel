package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.tech.DiscardTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DiscardChop : HGroupTech, DiscardTech() {
    override val name = "Discard Chop"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        return HGroupCommon.getChop(teammate.hand).index == slotIndex
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<DiscardAction> {
        return if (playerPOV.globallyAvailableInfo.clueTokens < 8) {
            return setOf(
                DiscardAction(
                    playerId = playerPOV.getOwnPlayerId(),
                    slotIndex = playerPOV.getOwnChop().index
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

    override fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
