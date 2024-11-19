package eelst.ilike.engine.player

import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    override val globallyAvailableInfo: GloballyAvailableInfo,
    override val ownHand: OwnHand,
    override val teammates: Set<Teammate>,
) : PlayerPOV {
    override fun getOwnKnownCards(): List<HanabiCard> {
        return ownHand.getKnownCards()
    }

    override fun getOwnKnownSlots(): Set<KnownSlot> {
        return ownHand.getKnownSlots()
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        val knownSlots = getOwnKnownSlots()
        return knownSlots.filter { globallyAvailableInfo.isImmediatelyPlayable(it.card) }.toSet()
    }

    override fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean{
        TODO()
    }

    override fun forEachTeammate(action: (teammate: Teammate) -> Unit) {
        return teammates.forEach(action)
    }
}
