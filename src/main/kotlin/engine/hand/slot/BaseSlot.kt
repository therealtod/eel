package eelst.ilike.engine.hand.slot

import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class BaseSlot(
    private val globallyAvailableInfo: SlotMetadata,
    private val possibleIdentities: Set<HanabiCard>
): Slot {
    override val index = globallyAvailableInfo.index
    override val positiveClues = globallyAvailableInfo.positiveClues
    override val negativeClues = globallyAvailableInfo.negativeClues

    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    override fun getPossibleIdentities(): Set<HanabiCard> {
        return possibleIdentities
    }

    override fun getGloballyAvailableInfo(): SlotMetadata {
        return globallyAvailableInfo
    }
}
