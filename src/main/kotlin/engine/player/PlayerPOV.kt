package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.GloballyAvailableInfoImpl
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant


interface PlayerPOV {
    val globallyAvailableInfo: GloballyAvailableInfo

    fun getOwnPlayerId(): PlayerId
    fun getOwnKnownCards(): List<HanabiCard>
    fun getOwnKnownSlots(): Set<KnownSlot>
    fun getOwnKnownPlayableSlots(): Set<Slot>
    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean
    fun forEachTeammate(action: (teammate: Teammate) -> Unit)
    fun getHand(playerId: PlayerId): InterpretedHand
    fun getTeammate(teammatePlayerId: PlayerId): Teammate
    fun getTeammates(): Set<Player<InterpretedSlot>>
    fun getOwnSlot(slotIndex: Int): OwnSlot
    fun getOwnChop(): OwnSlot
    fun getCardsKnownByTeammate(playerId: PlayerId): List<HanabiCard>
}
