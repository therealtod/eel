package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.PlayerId

interface PlayerKnowledge {
    fun getSlotKnowledge(playerId: PlayerId, slotIndex: Int): SlotKnowledge

    fun getHandKnowledge(playerId: PlayerId): HandKnowledge

    fun getKnowledgeAccessibleTo(playerId: PlayerId): PlayerKnowledge
}
