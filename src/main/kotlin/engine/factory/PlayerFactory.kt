package eelst.ilike.engine.factory

import eelst.ilike.engine.OwnSlot

import eelst.ilike.engine.PersonalKnowledge
import eelst.ilike.engine.Teammate
import eelst.ilike.engine.impl.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

object PlayerFactory {
    fun createActivePlayer(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
        otherPlayersKnowledge: Map<PlayerId, PersonalKnowledge>,
        teammatesHands: Map<PlayerId, TeammateHand>
    ): ActivePlayer {
        val thisPlayerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val numberOfPlayers = globallyAvailableInfo.players.size
        val activePlayerIndex = thisPlayerGlobalInfo.playerIndex
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }.values.map { playerInfo->
            createTeammate(
                playerId = playerInfo.playerId,
                playerIndex = playerInfo.playerIndex,
                globallyAvailableInfo = globallyAvailableInfo,
                personalKnowledge = otherPlayersKnowledge[playerInfo.playerId]!!,
                numberOfPlayers = numberOfPlayers,
                activePlayerIndex = activePlayerIndex,
                hand = teammatesHands[playerInfo.playerId]!!,
            )
        }.toSet()


        return ActivePlayer(
            playerId = playerId,
            playerIndex = activePlayerIndex,
            globallyAvailableInfo = globallyAvailableInfo,
            teammates = teammates,
            personalKnowledge = personalKnowledge,
        )
    }

    fun createTeammate(
        playerId: PlayerId,
        playerIndex: Int,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
        numberOfPlayers: Int,
        activePlayerIndex: Int,
        hand: TeammateHand,
    ): Teammate {
        val playerInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        return Teammate(
            playerId = playerInfo.playerId,
            playerIndex = playerIndex,
            seatsGap = (numberOfPlayers- activePlayerIndex + playerInfo.playerIndex).mod(numberOfPlayers),
            globallyAvailableInfo = globallyAvailableInfo,
            hand = hand,
            personalKnowledge = personalKnowledge,
        )
    }
}