package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

interface PlayerKnowledge {
    /**
     * Which cards the player can see on the table. Including playing stacks, trash pile, and teammates' hands
     */
    fun getVisibleCards(): List<HanabiCard>

    /**
     * A [Map] associating each slot index to the knowledge associate to it
     */
    fun getOwnHandKnowledge(): HandKnowledge

    fun getTeammateHandKnowledge(playerId: PlayerId): HandKnowledge
}
