package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant

sealed class Finesse : IndirectPlayClue() {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech !is DirectPlayClue && otherTech !is Prompt
    }
}
