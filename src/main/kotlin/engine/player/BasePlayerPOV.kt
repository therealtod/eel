package eelst.ilike.engine.player

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.convention.tech.PlayTech
import eelst.ilike.engine.hand.slot.FullEmpathySlot
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.*
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.card.HanabiCard

open class BasePlayerPOV(
    playerId: PlayerId,
    hand: Hand,
    private val gameData: GameData,
    private val personalKnowledge: PlayerPersonalKnowledge,
    private val teammates: Map<PlayerId, Teammate>,
) : GameFromPlayerPOV, Teammate(
    playerMetadata = gameData.getPlayerMetadata(playerId),
    hand = hand
) {
    private val globallyAvailablePlayerInfo = gameData.getPlayerMetadata(playerId)
    private val myself = Myself(
        playerMetadata = gameData.getPlayerMetadata(playerId),
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

    override fun getPlayers(): Map<PlayerId, Player> {
        return teammates + Pair(playerId, myself)
    }

    override fun getPlayer(playerId: PlayerId): Player {
        TODO("Not yet implemented")
    }

    override fun getPlayerMetadata(playerId: PlayerId): PlayerMetadata {
        TODO("Not yet implemented")
    }

    override fun getPlayerMetadata(playerIndex: Int): PlayerMetadata {
        TODO("Not yet implemented")
    }

    override fun getTeammate(teammatePlayerId: PlayerId): Teammate {
        return teammates[teammatePlayerId]
            ?: throw IllegalArgumentException("I can't see any teammate with id $teammatePlayerId")
    }

    override fun getSeatsGapFrom(teammate: Teammate): Int {
        return GameUtils.getSeatsGap(
            playerIndex1 = globallyAvailablePlayerInfo.playerIndex,
            playerIndex2 = teammate.playerIndex,
            gameData.numberOfPlayers,
        )
    }

    override fun getLegalActions(conventionSet: ConventionSet): Collection<ConventionalAction> {
        return getCandidateActions(conventionSet.getTechs()).toSet()
    }

    override fun getPersonalKnowledge(): PlayerPersonalKnowledge {
        return personalKnowledge
    }

    override fun getVisibleCards(): List<HanabiCard> {
        val cardsOnPlayingStacks = gameData.getCardsOnStacks()
        val cardsInTrash = gameData.trashPile.cards
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

    override fun getPlayerPOV(playerId: PlayerId): GameFromPlayerPOV {
        return if (playerId == myself.playerId) {
            this
        } else {
            getTeammate(playerId).getPOV(this)
        }
    }

    override fun getAsPlayer(): Teammate {
        return myself
    }


    override fun getGameData(): GameData {
        return gameData
    }

    override fun getAfter(drawAction: DrawAction, newSlot: Slot): GameFromPlayerPOV {
        TODO("Not yet implemented")
    }

    override fun getAfter(
        playAction: PlayAction,
        playedCard: HanabiCard,
        isStrike: Boolean,
        conventionSet: ConventionSet
    ): GameFromPlayerPOV {
        TODO("Not yet implemented")
    }

    override fun getAfter(
        discardAction: DiscardAction,
        discardedCard: HanabiCard,
        conventionSet: ConventionSet
    ): GameFromPlayerPOV {
        TODO("Not yet implemented")
    }

    override fun getAfter(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        conventionSet: ConventionSet
    ): GameFromPlayerPOV {
        TODO("Not yet implemented")
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