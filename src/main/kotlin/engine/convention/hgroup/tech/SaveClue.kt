package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.Suite


sealed class SaveClue(name: String, val appliesTo: Set<Suite>)
    : HGroupTech<ClueAction>(
    name = name,
)
