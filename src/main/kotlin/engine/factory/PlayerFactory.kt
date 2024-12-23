package eelst.ilike.engine.factory

import eelst.ilike.engine.convention.hgroup.strategy.HGroupGameStateEvaluator
import eelst.ilike.engine.strategy.BruteForceActionSelectionStrategy
import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerKnowledge
import eelst.ilike.game.GameData
import eelst.ilike.game.PlayerMetadata
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand

object PlayerFactory {
    fun createPlayer(
        metadata: PlayerMetadata,
        personalKnowledge: PlayerKnowledge,
        hand: Hand,
    ): Teammate {
        return Teammate(
            playerMetadata = metadata,
            hand = hand,
        )
    }

    fun createPlayerPOV(
        playerId: PlayerId,
        gameData: GameData,
        personalKnowledge: PlayerKnowledge,
        playersHands: Map<PlayerId, Hand>
    ): GameFromPlayerPOV {
        val players = playersHands.mapValues {
            createPlayer(
                metadata = gameData.getPlayerMetadata(it.key),
                personalKnowledge = personalKnowledge.getKnowledgeAccessibleTo(it.key),
                hand = it.value
            )
        }


        return BasePlayerPOV(
            playerId = playerId,
            gameData = gameData,
            personalKnowledge = personalKnowledge,
            teammates = players.minus(playerId),
            hand = playersHands[playerId]!!,
            actionSelectionStrategy = BruteForceActionSelectionStrategy(evaluator = HGroupGameStateEvaluator())
        )
    }
}
