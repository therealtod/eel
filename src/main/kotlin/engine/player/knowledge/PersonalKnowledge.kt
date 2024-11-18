package eelst.ilike.engine.player.knowledge


interface PersonalKnowledge{
    fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge
}
