package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.hgroup.HGroupCommon.validatePrompt
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.*

object SimplePrompt : Prompt(
    name = "Prompt",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
            val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
            val connectingCards = if (stack.isEmpty()) {
                card.getPrerequisiteCards()
            } else {
                card.suite.getCardsBetween(stack.currentCard(), card)
            }
            return validatePrompt(connectingCards.toSet(), playerPOV)
        }
        return false
    }


    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.hand.forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot.index, playerPOV))
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            slot = teammate.hand.getSlot(slot.index),
                            teammate = teammate,
                        )
                    )
            }
        }
        return actions.toSet()
    }


}
