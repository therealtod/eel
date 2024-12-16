package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
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

    fun hasCardOnFinessePosition(card: HanabiCard, engineHandlerPlayer: EngineHandlerPlayer, activePlayer: ActivePlayer): Boolean {
        if (hasFinessePosition(engineHandlerPlayer, activePlayer)) {
            val finessePosition = getFinessePosition(engineHandlerPlayer, activePlayer)
            return finessePosition.matches { _, identity ->
                identity == card
            }
        } else return false
    }

    private fun hasFinessePosition(engineHandlerPlayer: EngineHandlerPlayer, activePlayer: ActivePlayer): Boolean {
        return engineHandlerPlayer.hand.any { !isSlotTouched(it.index, engineHandlerPlayer.hand, activePlayer)}
    }

    private fun getFinessePosition(engineHandlerPlayer: EngineHandlerPlayer, activePlayer: ActivePlayer): Slot {
        return engineHandlerPlayer.hand.first { !isSlotTouched(it.index, engineHandlerPlayer.hand, activePlayer) }
    }
}
