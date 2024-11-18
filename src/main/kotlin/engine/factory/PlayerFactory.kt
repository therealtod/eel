package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createActivePlayer(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
        otherPlayersKnowledge: Map<PlayerId, PersonalKnowledge>,
        teammatesHands: Map<PlayerId, TeammateHand>
    ): ActivePlayerPOV {
        val thisPlayerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val numberOfPlayers = globallyAvailableInfo.players.size
        val activePlayerIndex = thisPlayerGlobalInfo.playerIndex
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }.values.map { playerInfo ->
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
        val slots = createOwnSlots(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
        )
        val hand = OwnHand(slots = slots.toSet())

        return ActivePlayerPOV(
            playerId = playerId,
            playerIndex = activePlayerIndex,
            hand = hand,
            globallyAvailableInfo = globallyAvailableInfo,
            teammates = teammates,
        )
    }

    private fun createTeammate(
        playerId: PlayerId,
        playerIndex: Int,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
        numberOfPlayers: Int,
        activePlayerIndex: Int,
        hand: TeammateHand,
    ): Teammate {
        val playerInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val ownSlots = createOwnSlots(
            playerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
        )
        return Teammate(
            playerId = playerInfo.playerId,
            playerIndex = playerIndex,
            seatsGap = (numberOfPlayers - activePlayerIndex + playerInfo.playerIndex).mod(numberOfPlayers),
            globallyAvailableInfo = globallyAvailableInfo,
            hand = hand,
            ownHand = OwnHand(ownSlots)
        )
    }

    private fun createOwnSlots(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
    ): Set<OwnSlot> {
        return (1..globallyAvailableInfo.handsSize).map {
            OwnSlot(
                globalInfo = globallyAvailableInfo.getPlayerInfo(playerId).hand.elementAt(it - 1),
                slotKnowledge = personalKnowledge.getKnowledgeAboutOwnSlot(it)
            )
        }.toSet()
    }
}