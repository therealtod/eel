package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction

/**
 * A [ConventionTech] associated with the action of playing
 */
interface PlayTech : ConventionTech {
    override fun matchesDiscard(
        discardAction: DiscardAction,
        gameState: GameState,
        currentKnowledge: TeamKnowledge
    ): Boolean {
        return false
    }

    override fun matchesClue(clueAction: ClueAction, gameState: GameState, currentKnowledge: TeamKnowledge): Boolean {
        return false
    }


    override fun getUpdatedKnowledge(discardAction: DiscardAction, currentKnowledge: TeamKnowledge): TeamKnowledge {
        return currentKnowledge
    }

    override fun getUpdatedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        currentKnowledge: TeamKnowledge
    ): TeamKnowledge {
        return currentKnowledge
    }
}
