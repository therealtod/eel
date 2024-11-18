package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.engine.hand.slot.OwnSlot

import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId

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

        val slots = (1..globallyAvailableInfo.handsSize).map {
            OwnSlot(
                globalInfo = globallyAvailableInfo.getPlayerInfo(playerId).hand.elementAt(it - 1),
                slotKnowledge = personalKnowledge.getKnowledgeAboutOwnSlot(it)
            )
        }
        val hand = OwnHand(slots = slots.toSet())

        return ActivePlayer(
            playerId = playerId,
            playerIndex = activePlayerIndex,
            hand = hand,
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