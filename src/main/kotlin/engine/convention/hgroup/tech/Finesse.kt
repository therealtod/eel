package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

sealed class Finesse(name: String) : IndirectPlayClue(name) {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech !is DirectPlayClue && otherTech !is Prompt
    }

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: VisibleTeammate, playerPOV: PlayerPOV): Boolean {
        if (hasFinessePosition(teammate, playerPOV)) {
            val finessePosition = getFinessePosition(teammate, playerPOV)
            return teammate.holdsCardInSlot(card, finessePosition.index)
        } else return false
    }

    private fun hasFinessePosition(teammate: Teammate, playerPOV: PlayerPOV): Boolean {
        return teammate.hand.any { !isSlotTouched(it.index, teammate.hand, playerPOV)}
    }

    private fun getFinessePosition(teammate: Teammate, playerPOV: PlayerPOV): Slot {
        return teammate.hand.first { !isSlotTouched(it.index, teammate.hand, playerPOV) }
    }
}
