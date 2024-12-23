package eelst.ilike.engine.factory

import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.GameUtils
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

object KnowledgeFactory {
    fun createEmptyPersonalKnowledge(playerPOV: GameFromPlayerPOV): PlayerKnowledge {
        return PlayerKnowledgeImpl(
            playerPOV.getPlayers().mapValues { createEmptyHandKnowledge() }.toMutableMap()
        )
    }

    fun createEmptyHandKnowledge(): HandKnowledge {
        return HandKnowledgeImpl()
    }
}