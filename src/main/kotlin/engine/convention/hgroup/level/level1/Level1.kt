package eelst.ilike.engine.convention.hgroup.level.level1

import eelst.ilike.engine.convention.hgroup.level.HGroupLevel
import eelst.ilike.engine.convention.hgroup.level.level1.tech.PlayKnownPlayable

data object Level1 : HGroupLevel(
    name = "Level 1",
    rank = 1,
    definedTechs = setOf(
        PlayKnownPlayable,
    )
)
