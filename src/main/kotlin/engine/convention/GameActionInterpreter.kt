package eelst.ilike.engine.convention

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction

object GameActionInterpreter {
    fun interpretPlay(
        playAction: PlayAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge,
        conventionSet: ConventionSet,
    ): Collection<ConventionTech> {
        val matchingTechs = conventionSet.getPlayTechs().filter {
            it.matchesPlay(
                playAction = playAction,
                globallyAvailableGameData = globallyAvailableGameData,
                currentKnowledge = currentKnowledge,
            )
        }
        return selectValidInterpretations(matchingTechs)
    }

    fun interpretDiscard(
        discardAction: DiscardAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge,
        conventionSet: ConventionSet,
        ): Collection<ConventionTech> {
        val matchingTechs = conventionSet.getDiscardTechs().filter {
            it.matchesDiscard(
                discardAction = discardAction,
                globallyAvailableGameData = globallyAvailableGameData,
                currentKnowledge = currentKnowledge,
            )
        }
       return selectValidInterpretations(matchingTechs)
    }

    fun interpretClue(
        clueAction: ClueAction,
        globallyAvailableGameData: GloballyAvailableGameData,
        currentKnowledge: TeamKnowledge,
        conventionSet: ConventionSet,
    ): Collection<ConventionTech> {
        val matchingTechs = conventionSet.getClueTechs().filter {
            it.matchesClue(
                clueAction = clueAction,
                globallyAvailableGameData = globallyAvailableGameData,
                currentKnowledge = currentKnowledge,
            )
        }
        return selectValidInterpretations(matchingTechs)
    }

    private fun selectValidInterpretations(matchingTechs: Collection<ConventionTech>): Collection<ConventionTech> {
        if (matchingTechs.isEmpty()) {
            return emptyList()
        } else {
            val techsSortedByInterpretationTier = matchingTechs.sortedBy { it.interpretationTier }
            val firstTech = techsSortedByInterpretationTier.first()
            val validInterpretations = techsSortedByInterpretationTier
                .takeWhile { it.interpretationTier == firstTech.interpretationTier }
            return validInterpretations
        }
    }
}
