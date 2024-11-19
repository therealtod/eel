package eelst.ilike.engine.player.knowledge

class PersonalKnowledgeImpl(private val slotKnowledge: Set<PersonalSlotKnowledge>) : PersonalKnowledge {
    override fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge {
        return slotKnowledge.elementAt(slotIndex - 1)
    }
}