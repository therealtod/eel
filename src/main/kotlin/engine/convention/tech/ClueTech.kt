package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

/**
 * A [ConventionTech] associated with the action of cluing
 */
interface ClueTech : ConventionTech {
    override fun matchesPlay(
        playAction: PlayAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean {
        return false
    }

    override fun matchesDiscard(
        discardAction: DiscardAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge
    ): Boolean {
        return false
    }

    override fun getUpdatedKnowledge(playAction: PlayAction, currentKnowledge: TeamKnowledge): TeamKnowledge {
        return currentKnowledge
    }

    override fun getUpdatedKnowledge(discardAction: DiscardAction, currentKnowledge: TeamKnowledge): TeamKnowledge {
        return currentKnowledge
    }
}
