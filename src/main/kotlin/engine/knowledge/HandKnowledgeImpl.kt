package eelst.ilike.engine.knowledge


class HandKnowledgeImpl(
    private val slotKnowledge: Map<Int, SlotKnowledge> = emptyMap()
) : HandKnowledge {
    override fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge {
        TODO()
    }
}
