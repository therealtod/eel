package eelst.ilike.engine.factory

import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailableInfoImpl
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createVisibleTeammate(
        teammateId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
    ): VisibleTeammate {
        val globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(teammateId)
        val hand = personalKnowledge.getVisibleHand(teammateId)
        return VisibleTeammate(
            globallyAvailablePlayerInfo = globallyAvailablePlayerInfo,
            personalKnowledge = personalKnowledge,
            hand = hand,
        )
    }

    fun createPlayerPOV(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
    ): PlayerPOV {
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }
            .map {
                createVisibleTeammate(
                    teammateId = it.key,
                    globallyAvailableInfo = globallyAvailableInfo,
                    personalKnowledge = personalKnowledge.accessibleTo(playerId),
                )
            }

        return PlayerPOVImpl(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            teammates = teammates.toSet()
        )
    }

    fun createPOVProjectionAsTeammate(
        playerPOV: PlayerPOV,
    ): POVProjectionAsTeammate {
        val globallyAvailableInfo = playerPOV.globallyAvailableInfo
        val playerInfo = globallyAvailableInfo.getPlayerInfo(playerPOV.getOwnPlayerId())
        return POVProjectionAsTeammate(
            globallyAvailableInfo = globallyAvailableInfo,
            globallyAvailablePlayerInfo = playerInfo,
            personalKnowledge = playerPOV.getPersonalKnowledge(),
            hand = playerPOV.getOwnHand()
        )
    }
}
