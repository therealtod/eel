package eelst.ilike.game

import eelst.ilike.engine.InterpretedSlot
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.entity.card.HanabiCard

data class VisibleSlot(
    val globalInfo: GloballyAvailableSlotInfo,
    val impliedIdentities: Set<HanabiCard>,
    private val card: HanabiCard
): InterpretedSlot(
    globalInfo = globalInfo,
    impliedIdentities = impliedIdentities
) {
    override fun getCard() = card

    override fun isKnown(playerPOV: PlayerPOV): Boolean {
        TODO("Not yet implemented")
    }

    override fun getPossibleIdentities(playerPOV: PlayerPOV): Set<HanabiCard> {
        return setOf(card)
    }
}
