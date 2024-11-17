package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard


interface PlayerPOV {
    val playerId: PlayerId
    val globallyAvailableInfo: GloballyAvailableInfo
    val ownHand: OwnHand
    val teammates: Set<Teammate>

    fun getOwnKnownCards(): List<HanabiCard>
    fun getOwnKnownSlots(): Set<KnownSlot>
    fun getOwnKnownPlayableSlots(): Set<Slot>
    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean
    fun forEachTeammate(action: (teammate: Teammate) -> Unit)
    fun getHand(playerId: PlayerId): InterpretedHand
    fun getTeammate(teammateplayerId: PlayerId): Teammate
}
