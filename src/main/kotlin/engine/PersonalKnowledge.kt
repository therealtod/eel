package eelst.ilike.engine


interface PersonalKnowledge{
    fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge
}
