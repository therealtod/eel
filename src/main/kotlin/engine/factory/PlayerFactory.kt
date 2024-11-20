package eelst.ilike.engine.factory

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.OldActivePlayer
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.PlayerPOVImpl
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId

object PlayerFactory {
    fun createActivePlayer(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        playersSlotKnowledge: Map<PlayerId, Set<PersonalSlotKnowledge>>,
        teammatesHands: Map<PlayerId, TeammateHand>
    ): OldActivePlayer{
        val personalKnowledge = PersonalKnowledgeImpl(
            slotKnowledge = playersSlotKnowledge[playerId]!!,
            teammatesHands = teammatesHands
        )

        return createActivePlayer(
            activePlayerId = playerId,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            teammatesHands = teammatesHands,
            playersSlotKnowledge = playersSlotKnowledge,
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
        hand: InterpretedHand,
        seatsGap: Int,
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
            /*
            playerPOV = createPlayerPOV(
                playerId = teammateId,
                playerIndex = globallyAvailablePlayerInfo.playerIndex,
                globallyAvailableInfo = globallyAvailableInfo,
                ownHand = OwnHand(slots),
                teammatesHands = ,
            ),

             */
            hand = TODO(),
            seatsGap = seatsGap
        )
    }

    private fun createActivePlayer(
        activePlayerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalKnowledge: PersonalKnowledge,
        teammatesHands: Map<PlayerId, TeammateHand>,
        playersSlotKnowledge: Map<PlayerId, Set<PersonalSlotKnowledge>>,
    ): OldActivePlayer{
        val activePlayerGloballyAvailableInfo = globallyAvailableInfo.getPlayerInfo(activePlayerId)
        val slots = createOwnSlots(
            handSize = globallyAvailableInfo.handsSize,
            playerGlobalInfo = activePlayerGloballyAvailableInfo,
            personalKnowledge = personalKnowledge,
        )
        val ownHand = OwnHand(slots)

        return OldActivePlayer(
            playerId = activePlayerGloballyAvailableInfo.playerId,
            playerIndex = activePlayerGloballyAvailableInfo.playerIndex,
            globallyAvailableInfo = globallyAvailableInfo,
            /*
            playerPOV = createPlayerPOV(
                playerId = activePlayerGloballyAvailableInfo.playerId,
                playerIndex = activePlayerGloballyAvailableInfo.playerIndex,
                globallyAvailableInfo = globallyAvailableInfo,
                ownHand = ownHand,
                teammatesHands = teammatesHands + Pair(activePlayerId, ownHand),
                playersSlotKnowledge = playersSlotKnowledge,
            ),

             */
        )
    }

    fun createPlayerPOV(
        playerId: PlayerId,
        playerIndex: Int,
        globallyAvailableInfo: GloballyAvailableInfo,
        ownHand: OwnHand,
        teammatesHands: Map<PlayerId, InterpretedHand>,
        playersSlotKnowledge: Map<PlayerId, Set<PersonalSlotKnowledge>>,
    ): PlayerPOV {
        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }
            .map {
                createTeammate(
                    teammateId = it.key,
                    globallyAvailableInfo = globallyAvailableInfo,
                    personalKnowledge = PersonalKnowledgeImpl(
                        slotKnowledge = playersSlotKnowledge[it.key]!!,
                        teammatesHands = teammatesHands.minus(playerId) //TODO
                    ),
                    hand = teammatesHands[it.key]!!,
                    seatsGap = GameUtils.getSeatsGap(
                        playerIndex1 = playerIndex,
                        playerIndex2 = it.value.playerIndex,
                        numberOfPlayers = globallyAvailableInfo.numberOfPlayers),
                )
            }


        return PlayerPOVImpl(
            globallyAvailableInfo = globallyAvailableInfo,
            ownHand = ownHand,
            teammates = teammates.toSet()
        )
    }
}
