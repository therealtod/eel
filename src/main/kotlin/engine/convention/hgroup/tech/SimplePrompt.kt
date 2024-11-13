package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.validatePrompt
import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow

object SimplePrompt
    : Prompt(
    name = "Prompt",
    appliesTo = setOf(NoVarRed, NoVarYellow, NoVarGreen, NoVarBlue, NoVarPurple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            teammate.hand.forEach { slot ->
                val card = slot.getCard()
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
                    actions.addAll(candidateClues.filter { validatePrompt(
                        connectingCards = connectingCards.toSet(),
                        clue = it,
                        playerPOV,
                    ) }
                        .map { ConventionalAction(
                            action = it,
                            tech = this,
                        ) }
                    )
                }
            }
        }
        return actions.toSet()
    }
}