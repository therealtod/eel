package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.clue.Clue
import eelst.ilike.game.entity.suite.Suite


sealed class SaveClue(name: String, val appliesTo: Set<Suite>) : HGroupTech<Clue>(
    name = name,
    takesPrecedenceOver = setOf(
        DelayedPlayClue,
        DirectPlayClue,
        SimpleFinesse,
        SimplePrompt,
    )
)
