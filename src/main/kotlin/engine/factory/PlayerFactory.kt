package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.PlayerPOVImpl
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableInfoImpl
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createTeammate(
        teammateId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfoImpl,
        personalKnowledge: PersonalKnowledge,
        seatsGap: Int,
    ): Teammate {
        val globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(teammateId)
        val hand = personalKnowledge.getVisibleHand(teammateId)
        return Teammate(
            playerId = teammateId,
            playerIndex = globallyAvailablePlayerInfo.playerIndex,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            hand = hand,
            seatsGap = seatsGap,
        )
    }

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

    fun createPlayerPOV(
        playerId: PlayerId,
        playerIndex: Int,
        globallyAvailableInfo: GloballyAvailableInfoImpl,
        ownHand: OwnHand,
        personalKnowledge: PersonalKnowledge,
    ): PlayerPOV {
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }
            .map {
                createTeammate(
                    teammateId = it.key,
                    globallyAvailableInfo = globallyAvailableInfo,
                    personalKnowledge = personalKnowledge.accessibleTo(playerId),
                    seatsGap = GameUtils.getSeatsGap(
                        playerIndex1 = playerIndex,
                        playerIndex2 = it.value.playerIndex,
                        numberOfPlayers = globallyAvailableInfo.numberOfPlayers
                    ),
                )
            }

        return PlayerPOVImpl(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            ownHand = ownHand,
            teammates = teammates.toSet()
        )
    }
}
