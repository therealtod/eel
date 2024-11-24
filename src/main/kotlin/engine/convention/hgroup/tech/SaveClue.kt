package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.Suite


sealed class SaveClue(name: String, val appliesTo: Set<Suite>) : HGroupTech(
    name = name,
) {
    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue
    }
}
