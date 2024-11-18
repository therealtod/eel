package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.validatePrompt
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.entity.suite.*

object SimplePrompt
    : Prompt(
    name = "Prompt",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            teammate.hand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
                    val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
                    val connectingCards = if (stack.isEmpty()) {
                        card.getPrerequisiteCards()
                    } else {
                        card.suite.getCardsBetween(stack.currentCard(), card)
                    }

                    val candidateClues = getAllFocusingClues(
                        card = card,
                        slot = slot,
                        teammate = teammate,
                    )
                    actions.addAll(candidateClues.filter {
                        validatePrompt(
                            connectingCards = connectingCards.toSet(),
                            playerPOV,
                        )
                    }
                        .map {
                            ConventionalAction(
                                action = it,
                                tech = this,
                            )
                        }
                    )
                }
            }
        }
        return actions.toSet()
    }
}