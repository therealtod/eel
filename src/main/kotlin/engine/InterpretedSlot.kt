package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.Slot
import eelst.ilike.game.Utils
import eelst.ilike.game.entity.card.HanabiCard

abstract class InterpretedSlot(
    globalInfo: GloballyAvailableSlotInfo,
    private val impliedIdentities: Set<HanabiCard>
): Slot {
    override val index = globalInfo.index
    override val positiveClues = globalInfo.positiveClues
    override val negativeClues = globalInfo.negativeClues

    abstract fun isKnown(playerPOV: PlayerPOV): Boolean

    override fun getEmpathy(playerPOV: PlayerPOV): Set<HanabiCard> {
        return Utils.getCardEmpathy(
            visibleCards = playerPOV.getVisibleCards(),
            positiveClues = positiveClues,
            negativeClues = negativeClues,
            stacks = playerPOV.globallyAvailableInfo.playingStacks
        )
    }

    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    open fun isClued(): Boolean {
        return positiveClues.isNotEmpty()
    }

    abstract fun getPossibleIdentities(playerPOV: PlayerPOV): Set<HanabiCard>
}