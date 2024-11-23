package eelst.ilike.engine.player

import eelst.ilike.engine.factory.HandFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

abstract class ConventionsUsingPlayer(
    final override val playerId: PlayerId,
    final override val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val personalKnowledge: PersonalKnowledge,
) : Player<InterpretedHand> {
    override val ownHand = HandFactory.createOwnHand(
        handSize = globallyAvailableInfo.handsSize,
        playerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId),
        personalHandKnowledge = personalKnowledge.getOwnHandKnowledge(playerId)
    )

    fun getOwnKnownCards(): List<HanabiCard> {
        return getOwnKnownSlots().map { it.card }
    }

    fun getOwnKnownSlots(): Set<KnownSlot> {
        return ownHand.getKnownSlots()
    }

    fun knows(slotIndex: Int): Boolean {
        return getOwnSlot(slotIndex).isKnown()
    }

    fun getOwnSlot(slotIndex: Int): InterpretedSlot {
        return ownHand.getSlot(slotIndex)
    }

    abstract fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean
}
