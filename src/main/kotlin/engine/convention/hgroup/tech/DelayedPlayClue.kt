package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*

object DelayedPlayClue
    : IndirectPlayClue(
    name = "Delayed Play Clue",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()

        playerPOV.teammates.forEach { teammate ->
            teammate
                .ownHand
                .forEach { slot ->
                    val card = teammate.getCardAtSlot(slot.index)
                    if (!teammate.knows(slot.index) &&
                        playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) > 0 &&
                        connectingCardsAreKnown(card, playerPOV)
                    ) {
                        actions.addAll(
                            getAllFocusingActions(
                                card = card,
                                slot = slot,
                                teammate = teammate,
                            )
                        )
                    }
                }
        }
        return actions.toSet()
    }

    private fun connectingCardsAreKnown(card: HanabiCard, playerPOV: ActivePlayerPOV): Boolean {
        val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
        val missingCards = if (stack.isEmpty()) {
            card.getPrerequisiteCards().toSet()
        } else {
            stack.suite.getCardsBetween(stack.currentCard(), card)
        }
        return playerPOV.teamKnowsAllCards(missingCards)
    }
}
