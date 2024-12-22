package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction

interface PlayTech : ConventionTech {
    override fun matches(discardAction: DiscardAction, playerPOV: GameFromPlayerPOV): Boolean {
        return false
    }

    override fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: GameFromPlayerPOV): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(discardAction: DiscardAction, playerPOV: GameFromPlayerPOV): Knowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        playerPOV: GameFromPlayerPOV
    ): Knowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }
}
