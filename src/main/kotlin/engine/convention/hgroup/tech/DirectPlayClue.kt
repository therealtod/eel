package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.*

object DirectPlayClue : PlayClue(
    name = "Direct Play Clue",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        return !teammate.knows(slotIndex) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.ownHand.forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot.index, playerPOV)) {
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            slot = teammate.hand.getSlot(slot.index),
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
