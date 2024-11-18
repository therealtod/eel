package eelst.ilike.engine.player.knowledge

class PersonalKnowledgeImpl(private val slotKnowledge: Map<Int, PersonalSlotKnowledge>) : PersonalKnowledge {
    override fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge {
        return slotKnowledge.elementAt(slotIndex - 1)
    }
}