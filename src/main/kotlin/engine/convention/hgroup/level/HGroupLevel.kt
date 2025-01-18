package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.BaseConventionSet
import eelst.ilike.engine.convention.tech.ConventionTech

abstract class HGroupLevel(
    name: String,
    includes: Set<HGroupLevel> = emptySet(),
    definedTechs: Set<ConventionTech>,
    val rank: Int,
) : BaseConventionSet(
    name = name,
    includes = includes,
    definedTechs = definedTechs
)
