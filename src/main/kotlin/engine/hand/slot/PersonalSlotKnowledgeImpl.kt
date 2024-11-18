package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.entity.card.HanabiCard

class PersonalSlotKnowledgeImpl(
    private val impliedIdentities: Set<HanabiCard>,
    private val empathy: Set<HanabiCard>,
) : PersonalSlotKnowledge {
    override fun isClued(): Boolean {
        TODO("Not yet implemented")
    }

    override fun getPossibleSlotIdentities(): Set<HanabiCard> {
        return impliedIdentities.ifEmpty { empathy }
    }
}