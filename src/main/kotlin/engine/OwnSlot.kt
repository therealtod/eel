package eelst.ilike.engine

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.Utils
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class OwnSlot(
    private val impliedIdentities: Set<HanabiCard>,
    globalInfo: GloballyAvailableSlotInfo,
    private val suites: Set<Suite>
) : InterpretedSlot(globalInfo) {
    fun getPossibleIdentities(visibleCards: List<HanabiCard>): Set<HanabiCard> {
        return impliedIdentities
            .ifEmpty { Utils.getCardEmpathy(
                visibleCards = visibleCards,
                positiveClues = positiveClues,
                negativeClues = negativeClues,
                suites = suites,
            )
            }
    }

    fun hasKnownIdentity(card: HanabiCard): Boolean {
        TODO()
    }


    fun isKnown(visibleCards: List<HanabiCard>): Boolean {
        return getPossibleIdentities(visibleCards).size == 1
    }

    override fun isClued(): Boolean {
        return positiveClues.isNotEmpty() || impliedIdentities.isNotEmpty()
    }
}
