package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.suite.Suite

sealed class Prompt(
    name: String,
    appliesTo: Set<Suite>,
) : IndirectPlayClue(
    name = name,
    appliesTo = appliesTo,
    takesPrecedenceOver = setOf(SimpleFinesse),
)
