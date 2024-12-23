package eelst.ilike.engine.player.knowledge

interface HandKnowledge {
    fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge

    fun getSlotKnowledge(slotIndex: Int): SlotKnowledge

    fun asNotVisible(): HandKnowledge
}
