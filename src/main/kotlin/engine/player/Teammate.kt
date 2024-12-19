package eelst.ilike.engine.player


import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.factory.SlotFactory
import eelst.ilike.engine.hand.slot.BaseSlot
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.BaseHand
import eelst.ilike.game.entity.Slot

open class Teammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    override val hand: Hand,
) : Player {
    final override val playerId = globallyAvailablePlayerInfo.playerId
    override val playerIndex = globallyAvailablePlayerInfo.playerIndex

    override fun getSlots(): Set<Slot> {
        return hand.getSlots()
    }

    fun playsBefore(otherTeammate: Teammate, playerPOV: PlayerPOV): Boolean {
        return playerPOV.getSeatsGapFrom(this) < playerPOV.getSeatsGapFrom(otherTeammate)
    }

    fun getPOV(playerPOV: PlayerPOV): PlayerPOV {
        val playersHands = playerPOV.getTeammates()
            .associateBy { it.playerId }
            .minus(playerId)
            .mapValues { it.value.hand } +
                Pair(playerPOV.getOwnPlayerId(), playerPOV.getOwnHand()) +
                Pair(playerId, getHandFromPlayerPOV())
        return PlayerFactory.createPlayerPOV(
            playerId = playerId,
            game = playerPOV.game,
            personalKnowledge = playerPOV.getPersonalKnowledge().accessibleTo(playerId),
            playersHands = playersHands
        )
    }

    fun getHandFromPlayerPOV(): Hand {
        val slots = hand
            .getSlots()
            .map { it as BaseSlot }
            .map { SlotFactory.createSlot(
                activePlayerId = playerId,
                slotOwnerId = playerId,
                globallyAvailableSlotInfo = it.getGloballyAvailableInfo(),
                knowledge = it.knowledge,
                visibleIdentity = null,
            ) }
        return BaseHand(ownerId = playerId, slots = slots.toSet())
    }
}
