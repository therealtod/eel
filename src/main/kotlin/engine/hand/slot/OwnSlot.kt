package eelst.ilike.engine.hand.slot

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite

class OwnSlot(
    globalInfo: GloballyAvailableSlotInfo,
    private val slotKnowledge: PersonalSlotKnowledge,
) : InterpretedSlot(globalInfo) {
    fun getPossibleIdentities(visibleCards: List<HanabiCard>, suits: Set<Suite>): Set<HanabiCard> {
        return slotKnowledge.getImpliedIdentities().ifEmpty { getPossibleIdentities(visibleCards, suits) }
    }

    fun hasKnownIdentity(card: HanabiCard, visibleCards: List<HanabiCard>, suits: Set<Suite>): Boolean {
        return isKnown(visibleCards, suits) && getPossibleIdentities(visibleCards, suits).first() == card
    }

    fun isKnown(visibleCards: List<HanabiCard>, suits: Set<Suite>): Boolean {
        return getPossibleIdentities(visibleCards, suits).size == 1
    }
/*
    fun asKnown(): KnownSlot {
        require(isKnown()) {
            "Cannot get an unknown slot as known"
        }
        return KnownSlot(
            globallyAvailableInfo = globalInfo,
            card = getPossibleIdentities().first()
        )
    }

 */

    override fun isClued(): Boolean {
        return globalInfo.positiveClues.isNotEmpty()
    }

    override fun contains(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        return hasKnownIdentity(
            card = card,
            visibleCards = playerPOV.getVisibleCards(),
            suits = playerPOV.globallyAvailableInfo.suits,
        )
    }
}
