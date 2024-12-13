package eelst.ilike.engine.player


import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class Teammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    val personalKnowledge: PlayerPersonalKnowledge,
    override val hand: Hand,
) : Player {
    final override val playerId = globallyAvailablePlayerInfo.playerId
    override val playerIndex = globallyAvailablePlayerInfo.playerIndex

    abstract fun isPOVProjection(): Boolean
    abstract fun asVisible(): VisibleTeammate

    override fun getSlots(): Set<Slot> {
        return hand.getSlots()
    }

    fun playsBefore(otherTeammate: Teammate, playerPOV: PlayerPOV): Boolean {
        return playerPOV.getSeatsGapFrom(this) < playerPOV.getSeatsGapFrom(otherTeammate)
    }

    fun getOwnKnownCards(): List<HanabiCard> {
        TODO()
    }

    fun knowsIdentityOfOwnSlot(slotIndex: Int): Boolean {
        TODO()
    }

    fun getPOV(): PlayerPOV {
        TODO()
    }
}
