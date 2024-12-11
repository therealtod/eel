package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

data class KnownSlot(
    val globallyAvailableInfo: GloballyAvailableSlotInfo,
    val card: HanabiCard
) : InterpretedSlot(
    globalInfo = globallyAvailableInfo
) {
    override fun contains(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        return this.card == card
    }
}
