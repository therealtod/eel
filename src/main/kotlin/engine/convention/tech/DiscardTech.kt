package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.knowledge.PlayerKnowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.PlayAction

interface DiscardTech : ConventionTech {
    override fun matches(playAction: PlayAction, playerPOV: GameFromPlayerPOV): Boolean {
        return false
    }

    override fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: GameFromPlayerPOV): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(playAction: PlayAction, playerPOV: GameFromPlayerPOV): PlayerKnowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge(playerPOV)
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        playerPOV: GameFromPlayerPOV
    ): PlayerKnowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge(playerPOV)
    }
}
