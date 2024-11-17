package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech

interface ConventionSet {
    val name: String
    val includes: Set<ConventionSet>

    // fun getPlayTechs(): Set<ConventionTech>
    // fun getDiscardTechs(): Set<ConventionTech>
    // fun getClueTechs(): Set<ConventionTech>
    fun getTechs(): Set<ConventionTech>
}
