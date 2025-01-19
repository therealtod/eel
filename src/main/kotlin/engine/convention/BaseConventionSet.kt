package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ClueTech
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.tech.DiscardTech
import eelst.ilike.engine.convention.tech.PlayTech

abstract class BaseConventionSet(
    override val name: String,
    override val includes: Set<ConventionSet> = emptySet(),
    private val definedTechs: Set<ConventionTech>,
) : ConventionSet {
    override fun getTechs(): Set<ConventionTech> {
        return definedTechs + includes.flatMap { it.getTechs() }
    }

    override fun getPlayTechs(): List<ConventionTech> {
        return getTechs().filterIsInstance<PlayTech>()
    }

    override fun getDiscardTechs(): List<ConventionTech> {
        return getTechs().filterIsInstance<DiscardTech>()
    }

    override fun getClueTechs(): List<ConventionTech> {
        return getTechs().filterIsInstance<ClueTech>()
    }
}
