package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.slot.FullEmpathySlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    playerId: PlayerId,
    hand: Hand,
    override val globallyAvailableInfo: GloballyAvailableInfo,
    private val personalKnowledge: PlayerPersonalKnowledge,
    private val teammates: Set<Teammate>,
) : PlayerPOV {
    private val visibleTeammates = teammates.filterIsInstance<VisibleTeammate>().toSet()
    private val globallyAvailablePlayerInfo = globallyAvailableInfo.getPlayerInfo(playerId)
    private val myself = Myself(
        globallyAvailableInfo.getPlayerInfo(playerId),
        hand
    )

    override fun getHand(playerId: PlayerId): Hand {
        return if (playerId == myself.playerId) {
            myself.hand
        } else {
            getTeammate(playerId).hand
        }
    }

    override fun getOwnPlayerId(): PlayerId {
        return myself.playerId
    }

    override fun getOwnKnownCards(): List<HanabiCard> {
        val ownHandKnowledge = personalKnowledge.getOwnHandKnowledge(playerId = myself.playerId)
        return myself.hand.map {
            ownHandKnowledge.getKnowledge(it.index)
        }
            .filter { it.isSlotKnown() }
            .map { it.getPossibleSlotIdentities().first() }
    }

    override fun isSlotKnown(slotIndex: Int): Boolean {
        return personalKnowledge
            .getOwnHandKnowledge(myself.playerId).getKnowledge(slotIndex)
            .isSlotKnown()
    }

    override fun getOwnKnownPlayableSlots(): Set<Slot> {
        val knownSlots = getOwnKnownSlots()
        return knownSlots.filter { globallyAvailableInfo.isImmediatelyPlayable(it.knownIdentity) }.toSet()
    }

    override fun getOwnSlotPossibleIdentities(slotIndex: Int): Set<HanabiCard> {
        return personalKnowledge
            .getOwnHandKnowledge(myself.playerId).getKnowledge(slotIndex)
            .getPossibleSlotIdentities()
    }

    override fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean {
        return cards
            .all { card ->
                teammates.any { teammate ->
                    teammate.getPOV(this).getOwnKnownCards().contains(card)
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

    override fun getTeammate(teammatePlayerId: PlayerId): Teammate {
        return teammates.find { it.playerId == teammatePlayerId }
            ?: throw IllegalArgumentException("I can't see any teammate with id $teammatePlayerId")
    }

    override fun getSeatsGapFrom(teammate: Teammate): Int {
        return GameUtils.getSeatsGap(
            playerIndex1 = globallyAvailablePlayerInfo.playerIndex,
            playerIndex2 = teammate.playerIndex,
            globallyAvailableInfo.numberOfPlayers,
        )
    }

    override fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction> {
        return getCandidateActions(conventionSet.getTechs()).toSet()
    }

    override fun asTeammateOf(teammatePlayerId: PlayerId): Teammate {
        return PlayerFactory.createPOVProjectionAsTeammate(teammateId = teammatePlayerId, playerPOV = this)
    }

    override fun getPersonalKnowledge(): PlayerPersonalKnowledge {
        return personalKnowledge
    }

    override fun getOwnSlotEmpathy(slotIndex: Int): Set<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun getVisibleCards(): List<HanabiCard> {
        val cardsOnPlayingStacks = globallyAvailableInfo.getCardsOnStacks()
        val cardsInTrash = globallyAvailableInfo.trashPile.cards
        val teammatesSlots = teammates.flatMap {
            it.hand.getSlots()
        }
        val visibleTeammatesCards = teammatesSlots
            .filterIsInstance<VisibleSlot>()
            .map { it.knownIdentity }
        val ownFullEmpathyCards = myself.hand.getSlots()
            .filterIsInstance<FullEmpathySlot>()
            .map { it.knownIdentity }

        return cardsOnPlayingStacks + cardsInTrash + visibleTeammatesCards + ownFullEmpathyCards
    }

    private fun getCandidateActions(
        techs: Collection<ConventionTech>
    ): Collection<ConventionalAction> {
        return techs
            .flatMap { tech ->
                tech.getGameActions(this)
                    .map {
                        ConventionalAction(
                            action = it,
                            tech = tech,
                        )
                    }
            }
    }

    private fun getOwnKnownSlots(): Collection<KnownSlot> {
        return myself.hand.getSlots().filterIsInstance<KnownSlot>()
    }

    private fun prune(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        val overlappingGroups = actions.groupBy { it.action }
        return overlappingGroups.map { group ->
            group.value.fold(listOf(group.value.first())) { curr, next ->
                if (curr.any { it.tech.overrides(next.tech) })
                    curr
                else
                    curr + next
            }
        }.flatten()
            .toSet()
    }
}
