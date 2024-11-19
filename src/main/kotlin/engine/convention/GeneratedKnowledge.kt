package eelst.ilike.engine.convention

import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.PlayerId

interface GeneratedKnowledge{
    fun getKnowledgeAcquiredBy(playerId: PlayerId): PersonalKnowledge
}