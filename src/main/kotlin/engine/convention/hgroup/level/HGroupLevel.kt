package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.BaseConventionSet
import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

sealed class HGroupLevel(
    name: String,
    includes: Set<HGroupLevel> = emptySet(),
    playTechs: Set<ConventionTech<PlayAction>>  = emptySet(),
    discardTechs: Set<ConventionTech<DiscardAction>> = emptySet(),
    clueTechs: Set<ConventionTech<ClueAction>> = emptySet(),
    val rank: Int,
) : BaseConventionSet(
    name = name,
    includes = includes,
    playTechs = playTechs,
    discardTechs = discardTechs,
    clueTechs = clueTechs,
)
