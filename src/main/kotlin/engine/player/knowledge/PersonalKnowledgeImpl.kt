package eelst.ilike.engine.player.knowledge

class PersonalKnowledgeImpl(private val slots: Set<PersonalSlotKnowledge>): PersonalKnowledge {
    override fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge {
        return slots.elementAt(slotIndex - 1)
    }
}