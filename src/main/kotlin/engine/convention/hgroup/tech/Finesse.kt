package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
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

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: Teammate, playerPOV: GameFromPlayerPOV): Boolean {
        if (hasFinessePosition(teammate, playerPOV)) {
            val finessePosition = getFinessePosition(teammate, playerPOV)
            return finessePosition.matches { _, identity ->
                identity == card
            }
        } else return false
    }

    private fun hasFinessePosition(teammate: Teammate, playerPOV: GameFromPlayerPOV): Boolean {
        return teammate.hand.any { !isSlotTouched(it.index, teammate.hand, playerPOV)}
    }

    private fun getFinessePosition(teammate: Teammate, playerPOV: GameFromPlayerPOV): Slot {
        return teammate.hand.first { !isSlotTouched(it.index, teammate.hand, playerPOV) }
    }
}
