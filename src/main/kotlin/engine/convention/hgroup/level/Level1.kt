package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.hgroup.tech.*

data object Level1 : HGroupLevel(
    name = "Level 1",
    rank = 1,
    playTechs = setOf(
        PlayKnownPlayable,
    ),
    discardTechs = setOf(
        DiscardChop,
    ),
    clueTechs = setOf(
        CriticalSave,
        DelayedPlayClue,
        SimplePrompt,
        SimpleFinesse,
        DirectPlayClue,
        FiveSave,
        )
)
