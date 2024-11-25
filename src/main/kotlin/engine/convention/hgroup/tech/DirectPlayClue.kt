package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.*

object DirectPlayClue : PlayClue(
    name = "Direct Play Clue",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.ownHand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (!teammate.knows(slot.index) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0) {
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            card = card,
                            teammate = teammate,
                        )
                    )
                }
            }
        }
        return actions.toSet()
    }

    override fun overrides(otherTech: ConventionTech<ClueAction>): Boolean {
        return otherTech !is SaveClue && otherTech is PlayClue
    }
}
