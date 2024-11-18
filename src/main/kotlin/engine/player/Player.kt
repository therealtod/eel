package eelst.ilike.engine.player


import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

abstract class Player(
    val playerId: PlayerId,
    val playerIndex: Int,
    val globallyAvailableInfo: GloballyAvailableInfo,
    open val hand: InterpretedHand,
    personalKnowledge: PersonalKnowledge,
) {
    protected val ownHand: OwnHand

    init {
        val playerHandGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId).hand
        val slots = playerHandGlobalInfo.map {
            OwnSlot(
                globalInfo = playerHandGlobalInfo.elementAt(it.index - 1),
                slotKnowledge = personalKnowledge.getKnowledgeAboutOwnSlot(it.index)
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

