package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge

interface ClueTech : ConventionTech {
    fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean
    fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PlayerPersonalKnowledge

    override fun matches(action: ObservedAction, playerPOV: PlayerPOV): Boolean {
        return if (action is ObservedClue) {
            matchesClue(action, playerPOV)
        } else {
            false
        }
    }

    override fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
        return if (action is ObservedClue) {
            getGeneratedKnowledge(action, playerPOV)
        } else {
            KnowledgeFactory.createEmptyPersonalKnowledge()
        }
    }
}
