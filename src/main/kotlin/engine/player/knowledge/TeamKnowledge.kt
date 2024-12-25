package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

interface TeamKnowledge {
    //fun getSlotKnowledge(playerId: PlayerId, slotIndex: Int): SlotKnowledge

    fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge

    fun getAsSeenBy(playerId: PlayerId): TeamKnowledge
}
