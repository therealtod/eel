package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

class OwnSlot(
    globalInfo: GloballyAvailableSlotInfo,
    private val impliedIdentities: Set<HanabiCard>,
) : InterpretedSlot(globalInfo, impliedIdentities) {

    override fun getCard(): HanabiCard {
        TODO("Not yet implemented")
    }

    override fun getPossibleIdentities(playerPOV: PlayerPOV): Set<HanabiCard> {
        return impliedIdentities.ifEmpty { getEmpathy(playerPOV) }
    }

    override fun isKnown(playerPOV: PlayerPOV): Boolean {
        return getPossibleIdentities(playerPOV).size == 1
    }

    override fun isClued(): Boolean {
        return positiveClues.isNotEmpty() || impliedIdentities.isNotEmpty()
    }
}
