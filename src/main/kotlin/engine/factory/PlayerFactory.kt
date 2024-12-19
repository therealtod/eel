package eelst.ilike.engine.factory

import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.Game
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
        game: Game,
        personalKnowledge: PlayerPersonalKnowledge,
        playersHands: Map<PlayerId, Hand>
    ): PlayerPOV {
        val players = playersHands.mapValues {
            createPlayer(
                globallyAvailableInfo = game.getPlayer(it.key),
                personalKnowledge = personalKnowledge.accessibleTo(it.key),
                hand = it.value
            )
        }


        return PlayerPOVImpl(
            playerId = playerId,
            game = game,
            personalKnowledge = personalKnowledge,
            teammates = players.minus(playerId),
            hand = playersHands[playerId]!!
        )
    }
}
