package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.PlayerPOVImpl
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createActivePlayer(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        playersSlotKnowledge: Map<PlayerId, Set<PersonalSlotKnowledge>>,
        teammatesHands: Map<PlayerId, TeammateHand>
    ): ActivePlayer{
        val personalKnowledge = PersonalKnowledgeImpl(
            slotKnowledge = playersSlotKnowledge[playerId]!!,
            teammatesHands = teammatesHands
        )

        return createActivePlayer(
            activePlayerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
        )
    }

    fun createOwnSlots(
        handSize: Int,
        playerGlobalInfo: GloballyAvailablePlayerInfo,
        personalKnowledge: PersonalKnowledge,
    ): Set<OwnSlot> {
        return (1..handSize).map {
            OwnSlot(
                globalInfo = playerGlobalInfo.hand.elementAt(it - 1),
                slotKnowledge = personalKnowledge.getKnowledgeAboutOwnSlot(it)
            )
        }.toSet()
    }

    fun createTeammate(
        teammateId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
        hand: TeammateHand,
    ): Teammate {
        val globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(teammateId)
        val slots = createOwnSlots(
            handSize = globallyAvailableInfo.handsSize,
            playerGlobalInfo = globallyAvailableInfo.getPlayerInfo(teammateId),
            personalKnowledge = personalKnowledge,
        )
        return Teammate(
            playerId = teammateId,
            playerIndex = globallyAvailablePlayerInfo.playerIndex,
            hand = hand,
            ownHand = OwnHand(slots),
            personalKnowledge = personalKnowledge,
            globallyAvailableInfo = globallyAvailableInfo,
            seatsGap = TODO()
        )
    }

    private fun createActivePlayer(
        activePlayerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
    ): ActivePlayer{
        val activePlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(activePlayerId)
        val slots = createOwnSlots(
            handSize = globallyAvailableInfo.handsSize,
            playerGlobalInfo = activePlayerGloballyAvailableInfo,
            personalKnowledge = personalKnowledge,
        )

        return ActivePlayer(
            playerId = activePlayerGloballyAvailableInfo.playerId,
            playerIndex = activePlayerGloballyAvailableInfo.playerIndex,
            globallyAvailableInfo = globallyAvailableInfo,
            playerPOV = createPlayerPOV(
                globallyAvailableInfo = globallyAvailableInfo,
                ownHand = OwnHand(slots),
                personalKnowledge = personalKnowledge,
            ),
        )
    }

    fun createPlayerPOV(
        globallyAvailableInfo: GloballyAvailableInfo,
        ownHand: OwnHand,
        personalKnowledge: PersonalKnowledge,
    ): PlayerPOV {
        return PlayerPOVImpl(
            globallyAvailableInfo = globallyAvailableInfo,
            ownHand = ownHand,
            personalKnowledge = personalKnowledge
        )
    }
}
