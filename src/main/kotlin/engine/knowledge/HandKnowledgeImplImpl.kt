package eelst.ilike.engine.knowledge


class HandKnowledgeImplImpl(
    private val slotKnowledge: Map<Int, SlotKnowledge> = emptyMap()
) : HandKnowledge {
    override fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge {
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
