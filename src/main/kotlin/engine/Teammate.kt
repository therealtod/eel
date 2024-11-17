package eelst.ilike.engine

import eelst.ilike.engine.impl.OwnHand
import eelst.ilike.engine.impl.TeammateHand
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class Teammate(
    val playerId: PlayerId,
    val seatsGap: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    val hand: TeammateHand,
    val personalInfo: PersonalInfo,
    val visibleCards: List<HanabiCard>,
) {
    private val playerHandGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId).hand
    private val ownHand: OwnHand


    init {
        val slots = playerHandGlobalInfo.map {
            OwnSlot(
                impliedIdentities = personalInfo.getOwnSlotInfo(it.index).impliedIdentities,
                globalInfo = playerHandGlobalInfo.elementAt(it.index -1 ),
                suites = globallyAvailableInfo.suites
            )
        }
        ownHand = OwnHand(slots.toSet())
    }

    fun playsBefore(teammate: Teammate): Boolean {
        return seatsGap < teammate.seatsGap
    }

    fun getKnownCards(): List<HanabiCard> {
        return ownHand.getKnownCards(visibleCards)
    }

    fun knows(slotIndex: Int): Boolean {
        return ownHand.getSlot(slotIndex).isKnown(
            visibleCards = visibleCards,
        )
    }

    fun getSlotFromTeammatePOV(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

    fun getCardAtSlot(slotIndex: Int): HanabiCard {
        return hand.getSlot(slotIndex).card
    }
}