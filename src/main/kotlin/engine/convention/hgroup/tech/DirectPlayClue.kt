package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.suite.*

object DirectPlayClue : PlayClue(
    name = "Direct Play Clue",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.ownHand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (!teammate.knows(slot.index) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0) {
                    actions.addAll(
                        getAllFocusingActions(
                            card = card,
                            slot = slot,
                            hand = teammate.hand
                        )
                    )
                }
            }
        }
        return actions.toSet()
    }
}
