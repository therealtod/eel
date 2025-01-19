package eelst.ilike.engine.game

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.GameActionInterpreter
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.HanabiCard


data class KnowledgeAwareGameState(
    val gameState: GameState,
    private val teamKnowledge: TeamKnowledge,
    private val conventionSet: ConventionSet,
) : GameState by gameState {
    override fun getAfter(drawAction: DrawAction): GameState {
        val newGameState = gameState.getAfter(drawAction)
        val newTeamKnowledge = teamKnowledge.getAfterDraw(
            drawAction = drawAction,
            globallyAvailableGameData = globallyAvailableGameData,
        )
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(drawAction: DrawAction, card: HanabiCard): GameState {
        val newGameState = gameState.getAfter(drawAction, card)
        val newTeamKnowledge = teamKnowledge.getAfterDrawing(
            drawAction = drawAction,
            card = card,
            globallyAvailableGameData = globallyAvailableGameData
        )
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(playAction: PlayAction): GameState {
        val newGameState = gameState.getAfter(playAction)
        val validInterpretations = GameActionInterpreter.interpretPlay(
            playAction = playAction,
            globallyAvailableGameData = globallyAvailableGameData,
            currentKnowledge = teamKnowledge,
            conventionSet = conventionSet,
        )
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = TODO(),
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState {
        val newGameState = gameState.getAfter(playAction)

        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = TODO(),
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(discardAction: DiscardAction): GameState {
        val newGameState = gameState.getAfter(discardAction)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = TODO(),
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        val newGameState = gameState.getAfter(discardAction, discardedCard)
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = TODO(),
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Collection<Int>): GameState {
        val newGameState = gameState.getAfter(clueAction, touchedSlotsIndexes)
        val newTeamKnowledge = teamKnowledge.getAfterClueGiven(
            clueAction = clueAction,
            touchedSlotsIndexes = touchedSlotsIndexes,
            globallyAvailableGameData = globallyAvailableGameData,
        )
        return KnowledgeAwareGameState(
            gameState = newGameState,
            teamKnowledge = newTeamKnowledge,
            conventionSet = conventionSet,
        )
    }
}
