package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.GameAction

interface PlayTech : ConventionTech {
    override fun matches(discardAction: DiscardAction, playerPOV: PlayerPOV): Boolean {
        return false
    }

    override fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: PlayerPOV): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(discardAction: DiscardAction, playerPOV: PlayerPOV): Knowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        playerPOV: PlayerPOV
    ): Knowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }
}
