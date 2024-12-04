package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.hand.InterpretedHand
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

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: VisibleTeammate): Boolean {
        if (hasFinessePosition(teammate.getVisibleHand())) {
            val finessePosition = getFinessePosition(teammate.getVisibleHand())
            return teammate.holdsCardInSlot(card, finessePosition.index)
        } else return false
    }

    private fun hasFinessePosition(hand: InterpretedHand): Boolean {
        return hand.any { !it.isTouched() }
    }

    private fun getFinessePosition(hand: InterpretedHand): Slot {
        return hand.first { !it.isTouched() }
    }
}
