package eelst.ilike.engine

import eelst.ilike.engine.factory.HandFactory
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

abstract class BasePlayer(
    final override val playerId: PlayerId,
    final override val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val personalKnowledge: PersonalKnowledge,
) : Player<OwnHand> {
    fun getOwnKnownCards(): List<HanabiCard> {
        return getOwnKnownSlots().map { it.card }
    }

    fun getOwnKnownSlots(): Set<KnownSlot> {
        return ownHand.getKnownSlots()
    }

    fun getSlotFromPlayerPOV(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

    fun knows(slotIndex: Int): Boolean {
        return ownHand.getSlot(slotIndex).isKnown()
    }

    fun forEachTeammate(action: (teammate: BasePlayer) -> Unit) {
        return teammates.forEach(action)
    }

    fun playsBefore(otherPlayer: BasePlayer, activePlayerIndex: Int): Boolean {
        TODO()
    }

    abstract fun getSlots(): Set<InterpretedSlot>

    abstract fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean
}
