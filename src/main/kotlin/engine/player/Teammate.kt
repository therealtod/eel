package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

abstract class Teammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    personalKnowledge: PersonalKnowledge,
    open val hand: InterpretedHand,
) : Player {
    override val playerId = globallyAvailablePlayerInfo.playerId
    override val playerIndex = globallyAvailablePlayerInfo.playerIndex

    abstract fun isPOVProjection(): Boolean
    abstract fun asVisible(): VisibleTeammate

    fun getSlot(slotIndex: Int): InterpretedSlot {
        return hand.getSlot(slotIndex)
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
}
