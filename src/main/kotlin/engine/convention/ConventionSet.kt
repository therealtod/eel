package eelst.ilike.engine.convention

import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

interface ConventionSet {
    val name: String
    val includes: Set<ConventionSet>
    fun getPlayTechs(): Set<ConventionTech<PlayAction>>
    fun getDiscardTechs(): Set<ConventionTech<DiscardAction>>
    fun getClueTechs(): Set<ConventionTech<ClueAction>>
    fun getTechs(): Set<ConventionTech<*>>
}
