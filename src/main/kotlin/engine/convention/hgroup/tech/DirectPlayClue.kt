package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.entity.suite.*

object DirectPlayClue : PlayClue(
    name = "Direct Play Clue",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.teammates.forEach { teammate ->
            teammate.hand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (!teammate.knows(slot.index) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0) {
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
}
