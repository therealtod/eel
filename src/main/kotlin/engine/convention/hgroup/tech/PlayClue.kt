package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.clue.Clue
import eelst.ilike.game.entity.suite.Suite


sealed class PlayClue(name: String, appliesTo: Set<Suite>, takesPrecedenceOver: Set<HGroupTech<Clue>>) :
    ClueTech(name = name, appliesTo = appliesTo, takesPrecedenceOver = takesPrecedenceOver)
