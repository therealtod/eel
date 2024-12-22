package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

interface ClueTech : ConventionTech {
    override fun matches(playAction: PlayAction, playerPOV: GameFromPlayerPOV): Boolean {
        return false
    }

    override fun matches(discardAction: DiscardAction, playerPOV: GameFromPlayerPOV): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(playAction: PlayAction, playerPOV: GameFromPlayerPOV): Knowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }

    override fun getGeneratedKnowledge(discardAction: DiscardAction, playerPOV: GameFromPlayerPOV): Knowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }
}
