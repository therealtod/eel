package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.VisibleHand
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
            personalKnowledge = personalKnowledge,
            hand = hand,
        )
    }

    /*
    fun createTeammate(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        playerPersonalKnowledge: PlayerPersonalKnowledge,
        hand: Hand,
    ): Teammate {
        return if (playerPersonalKnowledge.canSee(playerId)) {
            createVisibleTeammate(
                teammateId = playerId,
                playerInfo = globallyAvailableInfo.getPlayerInfo(playerId),
                personalKnowledge = playerPersonalKnowledge.accessibleTo(playerId),
                hand = playerPersonalKnowledge.getVisibleHand(playerId)
            )
        } else {
            createPOVProjectionAsTeammate(
                playerId = playerId,
                globallyAvailableInfo = globallyAvailableInfo,
                personalKnowledge = playerPersonalKnowledge,
            )
        }
    }

     */

    fun createVisibleTeammate(
        teammateId: PlayerId,
        playerInfo: GloballyAvailablePlayerInfo,
        personalKnowledge: PlayerPersonalKnowledge,
        hand: VisibleHand,
    ): VisibleTeammate {
        return VisibleTeammate(
            globallyAvailablePlayerInfo = playerInfo,
            personalKnowledge = personalKnowledge,
            visibleHand = hand,
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

    fun createPOVProjectionAsTeammate(
        teammateId: PlayerId,
        playerPOV: PlayerPOV,
    ): POVProjectionAsTeammate {
        TODO()
    }

    fun createPOVProjectionAsTeammate(
        playerId: PlayerId,
        teammatePlayerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PlayerPersonalKnowledge,
        hand: Hand,
    ): POVProjectionAsTeammate {
        return POVProjectionAsTeammate(
            globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(playerId),
            personalKnowledge = personalKnowledge.accessibleTo(teammatePlayerId),
            hand = hand,
        )
    }

}
