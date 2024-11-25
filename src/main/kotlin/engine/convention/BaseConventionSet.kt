package eelst.ilike.engine.convention

import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

abstract class BaseConventionSet(
    override val name: String,
    override val includes: Set<ConventionSet> = emptySet(),
    private val playTechs: Set<ConventionTech<PlayAction>> = emptySet(),
    private val discardTechs: Set<ConventionTech<DiscardAction>> = emptySet(),
    private val clueTechs: Set<ConventionTech<ClueAction>> = emptySet(),
): ConventionSet {
    override fun getPlayTechs(): Set<ConventionTech<PlayAction>> {
        return playTechs + includes.flatMap { it.getPlayTechs()}
    }

    override fun getDiscardTechs(): Set<ConventionTech<DiscardAction>> {
        return discardTechs + includes.flatMap { it.getDiscardTechs() }
    }

    override fun getClueTechs(): Set<ConventionTech<ClueAction>> {
        return clueTechs + includes.flatMap { it.getClueTechs() }
    }

    override fun getTechs(): Set<ConventionTech<*>> {
        return getPlayTechs() + getDiscardTechs() + getClueTechs()
    }
}
