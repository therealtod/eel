package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.BaseConventionSet
import eelst.ilike.engine.convention.tech.ConventionTech

sealed class HGroupLevel(
    name: String,
    includes: Set<HGroupLevel> = emptySet(),
    // playTechs: Set<ConventionTech> = emptySet(),
    // discardTechs: Set<ConventionTech> = emptySet(),
    // clueTechs: Set<ConventionTech> = emptySet(),
    definedTechs: Set<ConventionTech>,
    val rank: Int,
) : BaseConventionSet(
    name = name,
    includes = includes,
    definedTechs = definedTechs,
    // playTechs = playTechs,
    // discardTechs = discardTechs,
    // clueTechs = clueTechs,
)
