package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.Suite

sealed class IndirectPlayClue(
    name: String,
    appliesTo: Set<Suite>,
    takesPrecedenceOver: Set<HGroupTech>,
) : PlayClue(name = name, appliesTo = appliesTo, takesPrecedenceOver = takesPrecedenceOver) {
    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech !is DirectPlayClue
    }
}
