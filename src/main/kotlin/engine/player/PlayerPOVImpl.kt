package eelst.ilike.engine.player

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    override val playerId: PlayerId,
    override val globallyAvailableInfo: GloballyAvailableInfo,
    override val ownHand: OwnHand,
    private val teammates: Map<PlayerId, Teammate>,
    private val pastTurnsSnapshots: List<PlayerPOV> = emptyList()
) : PlayerPOV {
    private val snapshots = pastTurnsSnapshots + this

    override fun getOwnSlot(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

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
                teammates.values.any { teammate ->
                    teammate.getOwnKnownCards().contains(card)
                }
            }
    }

    override fun forEachTeammate(action: (teammate: Teammate) -> Unit) {
        return teammates.values.forEach(action)
    }

    override fun getTeammate(teammateId: PlayerId): Teammate {
        return teammates[teammateId] ?: throw IllegalArgumentException(
            "From $playerId 's POV there is not teammate with id $teammateId"
        )
    }

    override fun getTeammates(): Set<Teammate> {
        return teammates.values.toSet()
    }

    override fun getTeammateHand(playerId: PlayerId): VisibleHand {
        return getTeammate(playerId).hand
    }

    override fun getPlayerHand(playerId: PlayerId): InterpretedHand {
        return if(playerId == this.playerId) {
            ownHand
        } else {
            return getTeammateHand(playerId)
        }
    }

    override fun getPreviousTurnPOV(): PlayerPOV {
        return snapshots.last()
    }
}
