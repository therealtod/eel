package eelst.ilike.engine


import eelst.ilike.engine.impl.OwnHand
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

abstract class Player(
    val playerId: PlayerId,
    val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    private val personalKnowledge: PersonalKnowledge,
) {
    protected val ownHand: OwnHand

    init {
        val playerHandGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId).hand
        val slots = playerHandGlobalInfo.map {
            OwnSlot(
                globalInfo = playerHandGlobalInfo.elementAt(it.index -1 ),
                slotKnowledge = personalKnowledge.getKnowledgeAboutOwnSlot(it.index - 1)
            )
        }
        ownHand = OwnHand(slots.toSet())
    }

    fun getKnownCards(): List<HanabiCard> {
        return ownHand.getKnownCards()
    }

    fun knows(slotIndex: Int): Boolean {
        return ownHand.getSlot(slotIndex).isKnown()
    }

    fun getSlotFromPlayerPOV(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }
}

