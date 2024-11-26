package eelst.ilike.engine.convention

import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.PlayerId

class GenerateKnowledgeImpl(
    private val knowledge: Map<PlayerId, PersonalKnowledge>
) : GeneratedKnowledge {
    override fun getKnowledgeAcquiredBy(playerId: PlayerId): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
