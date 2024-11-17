package eelst.ilike.engine.factory

import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PersonalInfo
import eelst.ilike.engine.Teammate
import eelst.ilike.engine.impl.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createActivePlayer(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalInfo: PersonalInfo,
    ): ActivePlayer {
        val thisPlayerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val mySlots = thisPlayerGlobalInfo.hand.map {
            OwnSlot(
                globalInfo = it,
                impliedIdentities = personalInfo.getOwnSlotInfo(it.index).impliedIdentities
            )
        }
        val hand = OwnHand(mySlots.toSet())
        val numberOfPlayers = globallyAvailableInfo.players.size
        val activePlayerIndex = thisPlayerGlobalInfo.playerIndex
        val handMap = globallyAvailableInfo.players.filterKeys { it != playerId }.mapValues {
            personalInfo.getTeammateHand(it.key)
        }

        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }.values.map { playerInfo->
            createTeammate(
                globallyAvailableInfo = globallyAvailableInfo,
                playerInfo = playerInfo,
                numberOfPlayers = numberOfPlayers,
                activePlayerIndex = activePlayerIndex,
                handMap = handMap,
            )
        }.toSet()

        val pov = ActivePlayerPOV(
            globallyAvailableInfo = globallyAvailableInfo,
            teammates = teammates,
            hand = hand
        )

        return ActivePlayer(
            playerId = playerId,
            playerIndex = activePlayerIndex,
            hand = hand,
            globallyAvailableInfo = globallyAvailableInfo,
            pov = pov
        )
    }

    fun createTeammate(
        globallyAvailableInfo: GloballyAvailableInfo,
        playerInfo: GloballyAvailablePlayerInfo,
        numberOfPlayers: Int,
        activePlayerIndex: Int,
        handMap: Map<PlayerId, TeammateHand>
    ): Teammate {
        val teammateVisibleHand = handMap[playerInfo.playerId]
            ?: throw IllegalArgumentException("No hand data on a player with id ${playerInfo.playerId}")
        return Teammate(
            playerId = playerInfo.playerId,
            seatsGap = (numberOfPlayers- activePlayerIndex + playerInfo.playerIndex).mod(numberOfPlayers),
            globallyAvailableInfo = globallyAvailableInfo,
            hand = teammateVisibleHand,
            playerPOV = TeammatePOV(
                globallyAvailableInfo = globallyAvailableInfo,
                teammates =
            )
        )
    }
}