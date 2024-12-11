package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.entity.card.HanabiCard

class PersonalSlotKnowledgeImpl(
    private val impliedIdentities: Set<HanabiCard>,
    private val empathy: Set<HanabiCard>,
) : PersonalSlotKnowledge {
    override fun getImpliedIdentities(): Set<HanabiCard> {
        return impliedIdentities.ifEmpty { empathy }
    }
}
