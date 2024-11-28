package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

interface ConventionSet {
    val name: String
    val includes: Set<ConventionSet>
    // fun getPlayTechs(): Set<ConventionTech>
    // fun getDiscardTechs(): Set<ConventionTech>
    // fun getClueTechs(): Set<ConventionTech>
    fun getTechs(): Set<ConventionTech>
}
