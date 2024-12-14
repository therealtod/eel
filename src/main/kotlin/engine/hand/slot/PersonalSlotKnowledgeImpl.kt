package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class PersonalSlotKnowledgeImpl(
    private val ownerId: PlayerId,
    private val slotIndex: Int,
    private val impliedIdentities: Set<HanabiCard>,
    private val empathy: Set<HanabiCard>,
) : PersonalSlotKnowledge {
    override fun getOwnerId(): PlayerId {
        return ownerId
    }

    override fun getSlotIndex(): Int {
        return slotIndex
    }

    override fun getPossibleSlotIdentities(): Set<HanabiCard> {
        return impliedIdentities.ifEmpty { empathy }
    }

    override fun isSlotKnown(): Boolean {
        return getPossibleSlotIdentities().size == 1
    }

    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        TODO("Not yet implemented")
    }
}
