package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*

data object DelayedPlayClue
    : IndirectPlayClue(
    name = "Delayed Play Clue",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()

        playerPOV.forEachTeammate { teammate ->
            teammate
                .ownHand
                .forEach { slot ->
                    val card = teammate.getCardAtSlot(slot.index)
                    if (!teammate.knows(slot.index) &&
                        playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) > 0 &&
                        connectingCardsAreKnown(card, playerPOV)
                    ) {
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

    private fun connectingCardsAreKnown(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
        val missingCards = if (stack.isEmpty()) {
            card.getPrerequisiteCards().toSet()
        } else {
            stack.suite.getCardsBetween(stack.currentCard(), card)
        }
        return playerPOV.teamKnowsAllCards(missingCards)
    }
}
