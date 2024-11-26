package eelst.ilike.engine.player.knowledge

class PersonalHandKnowledgeImpl(private val slotKnowledge: Set<PersonalSlotKnowledge>) : PersonalHandKnowledge {
    override fun getKnowledge(slotIndex: Int): PersonalSlotKnowledge {
        return slotKnowledge.elementAt(slotIndex - 1)
    }
}
