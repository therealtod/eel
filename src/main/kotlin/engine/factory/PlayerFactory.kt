package eelst.ilike.engine.factory

import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GameData
import eelst.ilike.game.PlayerMetadata
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand

object PlayerFactory {
    fun createPlayer(
        metadata: PlayerMetadata,
        personalKnowledge: PlayerPersonalKnowledge,
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
        personalKnowledge: PlayerPersonalKnowledge,
        playersHands: Map<PlayerId, Hand>
    ): PlayerPOV {
        val players = playersHands.mapValues {
            createPlayer(
                metadata = gameData.getPlayerMetadata(it.key),
                personalKnowledge = personalKnowledge.accessibleTo(it.key),
                hand = it.value
            )
        }


        return PlayerPOVImpl(
            playerId = playerId,
            gameData = gameData,
            personalKnowledge = personalKnowledge,
            teammates = players.minus(playerId),
            hand = playersHands[playerId]!!
        )
    }
}
