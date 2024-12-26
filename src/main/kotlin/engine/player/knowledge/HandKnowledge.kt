package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.entity.card.HanabiCard

interface HandKnowledge {
    fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge
    fun getSlotKnowledge(slotIndex: Int): SlotKnowledge
    fun asNotVisible(): HandKnowledge
    fun getVisibleCards(): List<HanabiCard>
    fun getKnownCards(): List<HanabiCard>
}
