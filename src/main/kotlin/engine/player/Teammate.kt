package eelst.ilike.engine.player

import eelst.ilike.engine.factory.HandFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

abstract class Teammate(
    globallyAvailablePlayerInfo: GloballyAvailablePlayerInfo,
    val personalKnowledge: PersonalKnowledge,
    open val hand: InterpretedHand,
) : Player {
    final override val playerId = globallyAvailablePlayerInfo.playerId
    override val playerIndex = globallyAvailablePlayerInfo.playerIndex
    private val handFromTeammatePOV = HandFactory.createOwnHand(
        handSize = TODO(),
        playerGlobalInfo = globallyAvailablePlayerInfo,
        personalHandKnowledge = personalKnowledge.getOwnHandKnowledge(playerId)
    )

    abstract fun isPOVProjection(): Boolean
    abstract fun asVisible(): VisibleTeammate

    fun getSlotFromTeammatePOV(slotIndex: Int): OwnSlot {
        return handFromTeammatePOV.getSlot(slotIndex)
    }

    fun playsBefore(otherTeammate: Teammate, playerPOV: PlayerPOV): Boolean {
        return playerPOV.getSeatsGapFrom(this) < playerPOV.getSeatsGapFrom(otherTeammate)
    }

    fun getOwnKnownCards(): List<HanabiCard> {
        return handFromTeammatePOV.getKnownCards()
    }

    fun knowsIdentityOfOwnSlot(slotIndex: Int): Boolean {
        return handFromTeammatePOV.getSlot(slotIndex).isKnown()
    }
}
