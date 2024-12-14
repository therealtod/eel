package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

interface PersonalSlotKnowledge: Knowledge {
    fun getOwnerId(): PlayerId
    fun getSlotIndex(): Int
    fun getPossibleSlotIdentities(): Set<HanabiCard>
    fun isSlotKnown(): Boolean
}
