package eelst.ilike.engine.factory

import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand

object PlayerFactory {
    fun createPlayer(
        globallyAvailableInfo: GloballyAvailablePlayerInfo,
        personalKnowledge: PlayerPersonalKnowledge,
        hand: Hand,
    ): Teammate {
        return Teammate(
            globallyAvailablePlayerInfo = globallyAvailableInfo,
            hand = hand,
        )
    }

    fun createPlayerPOV(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PlayerPersonalKnowledge,
        playersHands: Map<PlayerId, Hand>
    ): PlayerPOV {
        val players = playersHands.mapValues {
            createPlayer(
                globallyAvailableInfo = globallyAvailableInfo.getPlayerInfo(it.key),
                personalKnowledge = personalKnowledge.accessibleTo(it.key),
                hand = it.value
            )
        }


        return PlayerPOVImpl(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            teammates = players.minus(playerId),
            hand = playersHands[playerId]!!
        )
    }
}
