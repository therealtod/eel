package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.Suite


sealed class SaveClue: HGroupTech, ClueTech() {
    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue
    }
}
