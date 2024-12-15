package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

abstract class BaseSlot(
    private val globallyAvailableInfo: GloballyAvailableSlotInfo,
    val knowledge: PersonalSlotKnowledge,
): Slot {
    override val index = globallyAvailableInfo.index
    override val positiveClues = globallyAvailableInfo.positiveClues
    override val negativeClues = globallyAvailableInfo.negativeClues

    override fun isTouched(): Boolean {
        return positiveClues.isNotEmpty()
    }

    override fun getPossibleIdentities(): Set<HanabiCard> {
        return knowledge.getPossibleSlotIdentities()
    }

    override fun getGloballyAvailableInfo(): GloballyAvailableSlotInfo {
        return globallyAvailableInfo
    }

    override fun getUpdatedEmpathy(clueValue: ClueValue): Set<HanabiCard> {
        return knowledge.getEmpathy().filter { it.isTouchedBy(clueValue) }.toSet()
    }
}
