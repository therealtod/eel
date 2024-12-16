package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.knowledge.Knowledge

interface ClueTech : ConventionTech {
    fun matchesClue(action: ObservedClue, activePlayer: ActivePlayer): Boolean
    fun getGeneratedKnowledge(action: ObservedClue, activePlayer: ActivePlayer): Knowledge

    override fun matches(action: ObservedAction, activePlayer: ActivePlayer): Boolean {
        return if (action is ObservedClue) {
            matchesClue(action, activePlayer)
        } else {
            false
        }
    }

    override fun getGeneratedKnowledge(action: ObservedAction, activePlayer: ActivePlayer): Knowledge {
        return if (action is ObservedClue) {
            getGeneratedKnowledge(action, activePlayer)
        } else {
            KnowledgeFactory.createEmptyPersonalKnowledge()
        }
    }
}
