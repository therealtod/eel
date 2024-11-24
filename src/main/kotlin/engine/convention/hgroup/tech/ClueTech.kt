package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.Suite

abstract class ClueTech(
    name: String,
    takesPrecedenceOver: Set<HGroupTech>,
    val appliesTo: Set<Suite>,
) : HGroupTech(name, takesPrecedenceOver)
