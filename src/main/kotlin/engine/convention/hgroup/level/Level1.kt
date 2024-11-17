package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.hgroup.tech.*

data object Level1 : HGroupLevel(
    name = "Level 1",
    rank = 1,
    definedTechs = setOf(
        PlayKnownPlayable,
        DiscardChop,
        CriticalSave,
        DelayedPlayClue,
        SimplePrompt,
        SimpleFinesse,
        DirectPlayClue,
        FiveSave,
    )
)
