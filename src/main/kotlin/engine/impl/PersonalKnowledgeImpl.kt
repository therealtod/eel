package eelst.ilike.engine.impl

import eelst.ilike.engine.PersonalKnowledge
import eelst.ilike.engine.PersonalSlotKnowledge

class PersonalKnowledgeImpl(private val slots: Set<PersonalSlotKnowledge>): PersonalKnowledge {
    override fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge {
        return slots.elementAt(slotIndex - 1)
    }
}