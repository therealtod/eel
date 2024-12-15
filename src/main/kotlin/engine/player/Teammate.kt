package eelst.ilike.engine.player


import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

open class Teammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    personalKnowledge: PlayerPersonalKnowledge,
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
            .mapValues { it.value.hand } +
                Pair(playerPOV.getOwnPlayerId(), playerPOV.getHand(playerPOV.getOwnPlayerId()))
        return PlayerFactory.createPlayerPOV(
            playerId = playerId,
            globallyAvailableInfo = playerPOV.globallyAvailableInfo,
            personalKnowledge = playerPOV.getPersonalKnowledge().accessibleTo(playerId),
            playersHands = playersHands
        )
    }
}
