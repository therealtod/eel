package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.validatePrompt
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.*

object SimplePrompt : Prompt(
    name = "Prompt",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
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
                        playerPOV = playerPOV,
                        card = card,
                        teammate = teammate,
                    )
                    actions.addAll(candidateClues.filter {
                        validatePrompt(
                            connectingCards = connectingCards.toSet(),
                            playerPOV,
                        )
                    })
                }
            }
        }
        return actions.toSet()
    }
}
