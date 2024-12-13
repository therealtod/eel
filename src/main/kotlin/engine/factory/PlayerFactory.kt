package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createTeammate(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        playerPersonalKnowledge: PlayerPersonalKnowledge,
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
    ): PlayerPOV {
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }
            .map {
                createTeammate(
                    playerId = it.key,
                    globallyAvailableInfo = globallyAvailableInfo,
                    playerPersonalKnowledge = personalKnowledge
                )
            }

        val hand = HandFactory.createPlayerHand(
            playerId = playerId,
            handSize = globallyAvailableInfo.defaultHandsSize,
            personalKnowledge = personalKnowledge,
            globallyAvailableSlotInfo = globallyAvailableInfo.players[playerId]!!.hand
        )

        return PlayerPOVImpl(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            teammates = teammates.toSet(),
            hand = hand
        )
    }

    fun createPlayerPOV(
        teammateId: PlayerId,
        originalPlayerPOV: PlayerPOV
    ): PlayerPOV {
        val teammates = originalPlayerPOV
            .getTeammates() +
                createPOVProjectionAsTeammate(teammateId = teammateId, playerPOV = originalPlayerPOV)

        return PlayerPOVImpl(
            playerId = teammateId,
            globallyAvailableInfo = originalPlayerPOV.globallyAvailableInfo,
            personalKnowledge = originalPlayerPOV.getPersonalKnowledge().accessibleTo(teammateId),
            teammates = teammates.toSet(),
            hand = originalPlayerPOV.getTeammate(teammateId).hand
        )
    }

    fun createPOVProjectionAsTeammate(
        teammateId: PlayerId,
        playerPOV: PlayerPOV,
    ): POVProjectionAsTeammate {
        val globallyAvailableInfo = playerPOV.globallyAvailableInfo
        val playerInfo = globallyAvailableInfo.getPlayerInfo(playerPOV.getOwnPlayerId())
        return POVProjectionAsTeammate(
            globallyAvailableInfo = globallyAvailableInfo,
            globallyAvailablePlayerInfo = playerInfo,
            personalKnowledge = playerPOV.getPersonalKnowledge().accessibleTo(teammateId),
            hand = playerPOV.getHand(playerPOV.getOwnPlayerId())
        )
    }

    fun createPOVProjectionAsTeammate(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PlayerPersonalKnowledge,
    ): POVProjectionAsTeammate {
        val playerInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        return POVProjectionAsTeammate(
            globallyAvailableInfo = globallyAvailableInfo,
            globallyAvailablePlayerInfo = playerInfo,
            personalKnowledge = personalKnowledge,
            hand = TODO()
        )
    }
}
