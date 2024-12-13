package eelst.ilike.engine.factory

import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.SimpleHand
import eelst.ilike.game.entity.SimpleSlot

object PlayerFactory {
    fun createTeammate(): Teammate {
        TODO()
    }

    fun createVisibleTeammate(
        teammateId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PlayerPersonalKnowledge,
    ): VisibleTeammate {
        val globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(teammateId)
        TODO()
    }

    fun createPlayerPOV(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PlayerPersonalKnowledge,
    ): PlayerPOV {
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }
            .map {
                createVisibleTeammate(
                    teammateId = it.key,
                    globallyAvailableInfo = globallyAvailableInfo,
                    personalKnowledge = personalKnowledge.accessibleTo(playerId),
                )
            }

        val slots =  (0..<globallyAvailableInfo.defaultHandsSize).map {
            SimpleSlot(
                globallyAvailableSlotInfo = globallyAvailableInfo.players[playerId]!!.hand.elementAt(it)
            )
        }

        return PlayerPOVImpl(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            teammates = teammates.toSet(),
            hand = SimpleHand(playerId, slots.size, slots.toSet())
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
            personalKnowledge = TODO(),
            hand = playerPOV.getHand(playerPOV.getOwnPlayerId())
        )
    }
}
