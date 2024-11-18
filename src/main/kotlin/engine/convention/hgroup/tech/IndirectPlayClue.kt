package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.suite.Suite

sealed class IndirectPlayClue(
    name: String,
    appliesTo: Set<Suite>,
    takesPrecedenceOver: Set<HGroupTech>,
) : PlayClue(name = name, appliesTo = appliesTo, takesPrecedenceOver = takesPrecedenceOver)
