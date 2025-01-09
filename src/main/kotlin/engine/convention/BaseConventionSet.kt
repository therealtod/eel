package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech

abstract class BaseConventionSet(
    override val name: String,
    override val includes: Set<ConventionSet> = emptySet(),
    private val definedTechs: Set<ConventionTech>,
) : ConventionSet {
    override fun getTechs(): Set<ConventionTech> {
        return definedTechs + includes.flatMap { it.getTechs() }
    }
}
