package eelst.ilike.engine.knowledge


class InferredHandKnowledgeImpl(
    private val slotKnowledge: Map<Int, SlotKnowledge> = emptyMap()
) : InferredHandKnowledge {
    override fun integrateWith(otherKnowledge: InferredHandKnowledge): InferredHandKnowledge {
        slotKnowledge.keys.forEach {
            slotKnowledge[it]!!.integrateWith(otherKnowledge.getSlotKnowledge(it))
        }
        return this
    }

    override fun getSlotKnowledge(slotIndex: Int): SlotKnowledge {
        return slotKnowledge.getOrDefault(slotIndex, BaseSlotKnowledge())
    }

    override fun getKnownSlotIndexes(): Collection<Int> {
        return slotKnowledge.filter {
            it.value.hasFullEmpathy() || it.value.getImpliedIdentities().size == 1
        }.map { it.key }
    }

    /*
    override fun getFullEmpathyCards(): List<HanabiCard> {
        return slotKnowledge
            .values
            .filter { it.hasFullEmpathy() }
            .map { TODO() }
    }

    override fun getKnownCards(): List<HanabiCard> {
        TODO("Not yet implemented")
    }

     */
}
