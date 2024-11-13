package eelst.ilike.engine.convention.hgroup.level

import eelst.ilike.engine.convention.hgroup.tech.*

class Level1 : HGroupLevel(
    name = "Level 1",
    rank = 1,
    techs = setOf(
        CriticalSave,
        DelayedPlayClue,
        SimplePrompt,
        SimpleFinesse,
        DirectPlayClue,
        FiveSave,
        DiscardChop,
        PlayKnownPlayable,
    )
)
