package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.Utils
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class OwnSlot(
    private val impliedIdentities: Set<HanabiCard>,
    globalInfo: GloballyAvailableSlotInfo,
    visibleCards: List<HanabiCard>,
    suites: Set<Suite>
) : InterpretedSlot(globalInfo) {
    val possibleIdentities = impliedIdentities
        .ifEmpty { Utils.getCardEmpathy(
            visibleCards = visibleCards,
            positiveClues = positiveClues,
            negativeClues = negativeClues,
            suites = suites,
        )
        }


    fun isKnown(): Boolean {
        return possibleIdentities.size == 1
    }

    override fun isClued(): Boolean {
        return positiveClues.isNotEmpty() || impliedIdentities.isNotEmpty()
    }
}
