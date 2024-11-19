package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasCardOnFinessePosition
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.entity.suite.*

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.ownHand.forEach { slot ->
                val card = teammate.getCardAtSlot(slot.index)
                if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
                    if (playerPOV.getTeammates().any { otherTeammate ->
                            otherTeammate.playsBefore(teammate) &&
                                    hasCardOnFinessePosition(
                                        card = card.suite.cardBefore(card),
                                        teammate = otherTeammate,
                                    )
                        }) {
                        actions.addAll(
                            getAllFocusingActions(
                                card = card,
                                slot = slot,
                                hand = teammate.hand,
                            )
                        )
                    }
                }
            }
        }
        return actions.toSet()
    }
}