package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech

/**
 * Represents a set of [ConventionTech] that are agreed on by all the players participating in a game
 */
interface ConventionSet {
    /**
     * The common name of the [ConventionTech]
     */
    val name: String

    /**
     * Inherits all the [ConventionTech] from other [ConventionSet] instances
     */
    val includes: Set<ConventionSet>

    /**
     * @return all the [ConventionTech] instances associated to this [ConventionSet]
     */
    fun getTechs(): Set<ConventionTech>

    /**
     * @return all the [ConventionTech] that can result in the action of playing a card
     */
    fun getPlayTechs(): List<ConventionTech>
}
