package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.entity.card.HanabiCard

class HandKnowledgeImpl(
    private val slotKnowledge: MutableMap<Int, SlotKnowledge> = mutableMapOf()
): HandKnowledge {
    override fun integrateWith(otherKnowledge: HandKnowledge): HandKnowledge {
        slotKnowledge.keys.forEach {
            slotKnowledge[it]!!.integrateWith(otherKnowledge.getSlotKnowledge(it))
        }
        return this
    }

    override fun getSlotKnowledge(slotIndex: Int): SlotKnowledge {
        return slotKnowledge.getOrDefault(slotIndex, DefaultSlotKnowledge())
    }

    override fun asNotVisible(): HandKnowledge {
        val slotKnowledge = slotKnowledge.mapValues { slotKnowledge[it.key]!!.asNotVisible() }.toMutableMap()
        return HandKnowledgeImpl(slotKnowledge)
    }

    override fun getVisibleCards(): List<HanabiCard> {
        return slotKnowledge.values.filter { it.isVisible() }.map { it.getIdentity() }
    }
}
