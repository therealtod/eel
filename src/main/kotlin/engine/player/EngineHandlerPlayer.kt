package eelst.ilike.engine.player


import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.factory.SlotFactory
import eelst.ilike.engine.hand.slot.BaseSlot
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.SimpleHand
import eelst.ilike.game.entity.Slot

open class EngineHandlerPlayer(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    override val hand: Hand,
) : Player {
    final override val playerId = globallyAvailablePlayerInfo.playerId
    override val playerIndex = globallyAvailablePlayerInfo.playerIndex

    override fun getSlots(): Set<Slot> {
        return hand.getSlots()
    }

    fun playsBefore(otherEngineHandlerPlayer: EngineHandlerPlayer, activePlayer: ActivePlayer): Boolean {
        return activePlayer.getSeatsGapFrom(this) < activePlayer.getSeatsGapFrom(otherEngineHandlerPlayer)
    }

    fun getPOV(activePlayer: ActivePlayer): ActivePlayer {
        val playersHands = activePlayer.getTeammates()
            .associateBy { it.playerId }
            .minus(playerId)
            .mapValues { it.value.hand } +
                Pair(activePlayer.getOwnPlayerId(), activePlayer.getOwnHand()) +
                Pair(playerId, getHandFromPlayerPOV())
        return PlayerFactory.createActivePlayer(
            playerId = playerId,
            globallyAvailableInfo = activePlayer.globallyAvailableInfo,
            personalKnowledge = activePlayer.getPersonalKnowledge().accessibleTo(playerId),
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
        return SimpleHand(ownerId = playerId, slots = slots.toSet())
    }
}
