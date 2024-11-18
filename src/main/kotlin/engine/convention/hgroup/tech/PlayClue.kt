package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.suite.Suite


sealed class PlayClue(name: String, appliesTo: Set<Suite>, takesPrecedenceOver: Set<HGroupTech>) :
    ClueTech(name = name, appliesTo = appliesTo, takesPrecedenceOver = takesPrecedenceOver)
