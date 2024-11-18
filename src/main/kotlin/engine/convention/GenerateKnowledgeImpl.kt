package eelst.ilike.engine.convention

import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.game.PlayerId

class GenerateKnowledgeImpl(
    private val knowledge: Map<PlayerId, PersonalKnowledge>
): GeneratedKnowledge {
    override fun getKnowledgeAcquiredby(playerId: PlayerId): PersonalKnowledge {
        return knowledge.getOrDefault(
            playerId,
            PersonalKnowledgeImpl(
                slots = emptySet()
            )
        )
    }
}
