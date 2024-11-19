package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.action.GameAction
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
        return recognize(playerPOV)
            .map {
                ConventionalAction(
                    action = it,
                    tech = this,
                )
            }.toSet()
    }

    override fun recognize(playerPOV: ActivePlayerPOV): Set<GameAction> {
        val actions = mutableSetOf<GameAction>()

        playerPOV.teammates.forEach { teammate ->
            if (hasChop(teammate.hand)) {
                val chop = getChop(teammate.hand)
                val card = teammate.getCardAtSlot(chop.index)
                if (appliesTo.contains(card.suite) &&
                    card.rank != Rank.FIVE &&
                    playerPOV.globallyAvailableInfo.isCritical(card) &&
                    !isGloballyKnownPlayable(card, playerPOV)
                ) {
                    actions.addAll(
                        getAllFocusingClues(
                            card = card,
                            slot = chop,
                            teammate = teammate,
                        )
                    )
                }
            }
        }
        return actions
    }
}
