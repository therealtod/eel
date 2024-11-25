package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.Suite

sealed class Finesse(name: String, appliesTo: Set<Suite>, takesPrecedenceOver: Set<HGroupTech<ClueAction>>) :
    IndirectPlayClue(name = name, appliesTo = appliesTo, takesPrecedenceOver = takesPrecedenceOver) {
    override fun overrides(otherTech: ConventionTech<ClueAction>): Boolean {
        return otherTech !is SaveClue && otherTech !is DirectPlayClue && otherTech !is Prompt
    }
    }
