package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.clue.Clue
import eelst.ilike.game.entity.suite.Suite

abstract class ClueTech(
    name: String,
    takesPrecedenceOver: Set<HGroupTech<Clue>>,
    val appliesTo: Set<Suite>,
) : HGroupTech<Clue>(name, takesPrecedenceOver)
