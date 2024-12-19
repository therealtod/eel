package eelst.ilike.engine.player

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.slot.FullEmpathySlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.Game
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    playerId: PlayerId,
    hand: Hand,
    override val game: Game,
    private val personalKnowledge: PlayerPersonalKnowledge,
    private val teammates: Map<PlayerId, Teammate>,
) : PlayerPOV, Teammate(
    globallyAvailablePlayerInfo = game.getPlayer(playerId),
    hand = hand
) {
    private val globallyAvailablePlayerInfo = game.getPlayer(playerId)
    private val myself = Myself(
        globallyAvailablePlayerInfo = game.getPlayer(playerId),
        hand,
    )

    override fun getOwnHand(): Hand {
        return myself.hand
    }

    override fun getOwnPlayerId(): PlayerId {
        return myself.playerId
    }

    override fun getOwnKnownCards(): List<HanabiCard> {
        return myself.hand.getSlots().filterIsInstance<KnownSlot>().map { it.knownIdentity }
    }

    override fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean {
        return cards
            .all { card ->
                teammates.values.any { teammate ->
                    teammate.getPOV(this).getOwnKnownCards().contains(card)
                } ||
                        getOwnKnownCards().contains(card)
            }
    }

    override fun forEachTeammate(action: (teammate: Teammate) -> Unit) {
        return teammates.values.forEach(action)
    }

    override fun getTeammates(): Set<Teammate> {
        return teammates.values.toSet()
    }

    override fun getTeammate(teammatePlayerId: PlayerId): Teammate {
        return teammates[teammatePlayerId]
            ?: throw IllegalArgumentException("I can't see any teammate with id $teammatePlayerId")
    }

    override fun getSeatsGapFrom(teammate: Teammate): Int {
        return GameUtils.getSeatsGap(
            playerIndex1 = globallyAvailablePlayerInfo.playerIndex,
            playerIndex2 = teammate.playerIndex,
            game.numberOfPlayers,
        )
    }

    override fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction> {
        return getCandidateActions(conventionSet.getTechs()).toSet()
    }

    override fun getPersonalKnowledge(): PlayerPersonalKnowledge {
        return personalKnowledge
    }

    override fun getVisibleCards(): List<HanabiCard> {
        val cardsOnPlayingStacks = game.getCardsOnStacks()
        val cardsInTrash = game.trashPile.cards
        val teammatesSlots = teammates.values.flatMap {
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

    override fun getPlayerPOV(playerId: PlayerId): PlayerPOV {
        return if (playerId == myself.playerId) {
            this
        } else {
            getTeammate(playerId).getPOV(this)
        }
    }

    override fun getAsPlayer(): Teammate {
        return myself
    }

    override fun getAfter(
        action: ObservedAction,
        game: Game,
        techs: Collection<ConventionTech>,
    ): PlayerPOV {
        val knowledge = techs.map {
            it.getGeneratedKnowledge(action, this)
        }

        return PlayerFactory.createPlayerPOV(
            playerId = playerId,
            game = game,
            personalKnowledge = TODO(),
            playersHands = TODO()
        )
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
