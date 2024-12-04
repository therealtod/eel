package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableInfoImpl
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createVisibleTeammate(
        teammateId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfoImpl,
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

    /*
    fun createActivePlayer(
        activePlayerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfoImpl,
        personalKnowledge: PersonalKnowledge,
    ): ActivePlayer {
        val activePlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(activePlayerId)

        return ActivePlayer(
            playerId = activePlayerGloballyAvailableInfo.playerId,
            playerIndex = activePlayerGloballyAvailableInfo.playerIndex,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge
        )
    }


     */
    fun createPlayerPOV(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfoImpl,
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
}
