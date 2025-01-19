package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction

/**
 * A [ConventionTech] associated with the action of playing
 */
interface PlayTech : ConventionTech {
    override fun matchesDiscard(
        discardAction: DiscardAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean {
        return false
    }

    override fun matchesClue(
        clueAction: ClueAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean {
        return false
    }


    override fun getUpdatedKnowledge(
        discardAction: DiscardAction,
        currentKnowledge: TeamKnowledge
    ): TeamKnowledge {
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
