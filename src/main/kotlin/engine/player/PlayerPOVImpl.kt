package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    override val playerId: PlayerId,
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

    override fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean {
        return cards
            .all { card ->
                teammates.any { teammate ->
                    teammate.getOwnKnownCards().contains(card)
                }
            }
    }

    override fun forEachTeammate(action: (teammate: Teammate) -> Unit) {
        return teammates.forEach(action)
    }

    override fun getTeammate(teammateplayerId: PlayerId): Teammate {
        return teammates.find { it.playerId == teammateplayerId }
            ?: throw IllegalArgumentException("I can't see any teammate with id $teammateplayerId")
    }

    override fun getHand(playerId: PlayerId): InterpretedHand {
        return if (playerId == this.playerId) {
            ownHand
        } else getTeammate(playerId).hand
    }
}
