package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.hgroup.tech.HGroupTech

sealed class HGroupLevel(
    name: String,
    includes: Set<HGroupLevel> = emptySet(),
    techs: Set<HGroupTech<*>>,
    val rank: Int,
) : ConventionSet(
    name = name,
    includes = includes,
    techs = techs
)
