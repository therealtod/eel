package eelst.ilike.engine.game

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.knowledge.KnowledgeFactory
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.variant.Variant


data class KnowledgeAwareGameState(
    val gameState: GameState,
    private val teamKnowledge: TeamKnowledge,
    private val conventionSet: ConventionSet,
) : GameState by gameState {
    override val numberOfPlayers = players.size

    private val availableColors = globallyAvailableGameData.variant.getSuits().flatMap { it.getAssociatedColors() }
    private val availableRanks = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)

    override val defaultHandsSize = GameUtils.getHandSize(numberOfPlayers)

    override fun getPlayer(playerId: PlayerId): Player {
        return players[playerId]
            ?: throw IllegalArgumentException("No player with id: $playerId in this game")
    }

    override fun getAvailableClueValues(): Set<ClueValue> {
        return (availableColors + availableRanks).toSet()
    }

    override fun getAfter(drawAction: DrawAction): GameState {
        val newGameState = gameState.getAfter(drawAction)
        val newTeamKnowledge = teamKnowledge.getAfter(drawAction)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(drawAction: DrawAction, card: HanabiCard): GameState {
        val newGameState = gameState.getAfter(drawAction, card)
        val newTeamKnowledge = teamKnowledge.getAfter(drawAction, card)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(playAction: PlayAction): GameState {
        val newGameState = gameState.getAfter(playAction)
        val newTeamKnowledge = teamKnowledge.getAfter(playAction, conventionSet)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState {
        val newGameState = gameState.getAfter(playAction)
        val newTeamKnowledge = teamKnowledge.getAfter(playAction, conventionSet)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(discardAction: DiscardAction): GameState {
        val newGameState = gameState.getAfter(discardAction)
        val newTeamKnowledge = teamKnowledge.getAfter(discardAction, conventionSet)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        val newGameState = gameState.getAfter(discardAction, discardedCard)
        val newTeamKnowledge = teamKnowledge.getAfter(discardAction, discardedCard, conventionSet)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Collection<Int>): GameState {
        val newGameState = gameState.getAfter(clueAction, touchedSlotsIndexes)
        val newTeamKnowledge = teamKnowledge.getAfter(clueAction, touchedSlotsIndexes, conventionSet)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    val efficiency: Float = TODO()

}
