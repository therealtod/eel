package eelst.ilike.engine.player


import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.game.PlayerMetadata
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.Slot

open class Teammate(
    playerMetadata: PlayerMetadata,
    override val hand: Hand,
) : Player {
    final override val playerId = playerMetadata.playerId
    override val playerIndex = playerMetadata.playerIndex

    override fun getSlots(): List<Slot> {
        return hand.getSlots()
    }

    fun playsBefore(otherTeammate: Teammate, playerPOV: GameFromPlayerPOV): Boolean {
        return playerPOV.getSeatsGapFrom(this) < playerPOV.getSeatsGapFrom(otherTeammate)
    }

    fun getPOV(playerPOV: GameFromPlayerPOV): GameFromPlayerPOV {
        val playersHands = playerPOV.getTeammates()
            .associateBy { it.playerId }
            .minus(playerId)
            .mapValues { it.value.hand } +
                Pair(playerPOV.getOwnPlayerId(), playerPOV.getOwnHand()) +
                Pair(playerId, getHandFromPlayerPOV())
        return PlayerFactory.createPlayerPOV(
            playerId = playerId,
            gameData = playerPOV.getGameData(),
            personalKnowledge = playerPOV.getPersonalKnowledge().getAsSeenBy(playerId),
            slotData= TODO()
        )
    }

    fun getHandFromPlayerPOV(): Hand {
        /*
        val slots = hand
            .getSlots()
            .map { it as BaseSlot }
            .map { SlotFactory.createSlot(
                slotMetadata = it.getGloballyAvailableInfo(),
                knowledge = ,
                slotMetadata = it.getGloballyAvailableInfo(),
                knowledge = it.knowledge,
                visibleIdentity = null,
            ) }
        return BaseHand(slots = slots)
         */
        TODO()
    }
}
