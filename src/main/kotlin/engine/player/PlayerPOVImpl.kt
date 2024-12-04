package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.factory.HandFactory
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    playerId: PlayerId,
    override val globallyAvailableInfo: GloballyAvailableInfo,
    val personalKnowledge: PersonalKnowledge,
    private val teammates: Set<Teammate>,
) : PlayerPOV {
    private val visibleTeammates = teammates.filterIsInstance<VisibleTeammate>().toSet()
    private val globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(playerId)
    private val ownHand = HandFactory.createOwnHand(
        handSize = globallyAvailableInfo.defaultHandsSize,
        playerGlobalInfo = globallyAvailablePlayerInfo,
        personalHandKnowledge = personalKnowledge.getOwnHandKnowledge(playerId)
    )

    override fun getOwnPlayerId(): PlayerId {
        return globallyAvailablePlayerInfo.playerId
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
                teammates.any { teammate ->
                    teammate.getOwnKnownCards().contains(card)
                } ||
                        getOwnKnownCards().contains(card)
            }
    }

    override fun forEachVisibleTeammate(action: (teammate: VisibleTeammate) -> Unit) {
        return visibleTeammates.forEach(action)
    }

    override fun getTeammates(): Set<Teammate> {
        return teammates
    }

    override fun getVisibleTeammates(): Set<VisibleTeammate> {
        return visibleTeammates
    }

    override fun getTeammate(teammateplayerId: PlayerId): Teammate {
        return teammates.find { it.playerId == teammateplayerId }
            ?: throw IllegalArgumentException("I can't see any teammate with id $teammateplayerId")
    }

    override fun getOwnSlot(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

    override fun getHand(playerId: PlayerId): InterpretedHand {
        return if (playerId == getOwnPlayerId()) {
            ownHand
        } else getTeammate(playerId).hand
    }

    override fun getOwnHand(): OwnHand {
        return ownHand
    }

    override fun getHandFromTeammatePOV(teammatePlayerId: PlayerId): OwnHand {
        TODO()
    }

    override fun getSeatsGapFrom(teammate: Teammate): Int {
        return GameUtils.getSeatsGap(
            playerIndex1 = globallyAvailablePlayerInfo.playerIndex,
            playerIndex2 = teammate.playerIndex,
            globallyAvailableInfo.numberOfPlayers,
        )
    }

    override fun getLegalActions(conventionSet: ConventionSet) {
        TODO("Not yet implemented")
    }
}
