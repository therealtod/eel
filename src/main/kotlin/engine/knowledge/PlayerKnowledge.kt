package eelst.ilike.engine.knowledge

import eelst.ilike.game.entity.HanabiCard


interface PlayerKnowledge {
    /**
     * Get all the cards that the player can see on the table.
     * Including playing stacks, trash pile, and teammates' hands
     */
    fun getVisibleCards(): List<HanabiCard>

    /**
     * Get the knowledge that the player has about their own hand
     */
    fun getOwnHandKnowledge(): HandKnowledge

    /**
     * Get the identities of all the slots for which the hand owner has full empathy
     */
    fun getFullEmpathyCards(): List<HanabiCard>

    /**
     * Get a list of [HanabiCard] which the hand owner knows to have in their hand either by empathy or deduction
     */
    fun getKnownCards(): List<HanabiCard>
}
