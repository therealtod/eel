package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.getChop
import eelst.ilike.engine.convention.hgroup.HGroupHelper.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupHelper.isGloballyKnownPlayable
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Blue
import eelst.ilike.game.entity.suite.Green
import eelst.ilike.game.entity.suite.Purple
import eelst.ilike.game.entity.suite.Red
import eelst.ilike.game.entity.suite.Yellow

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()

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
                        getAllFocusingActions(
                            card = card,
                            slot = chop,
                            teammate = teammate,
                        )
                    )
                }
            }
        }
        return actions.toSet()
    }
}
