package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.GeneratedKnowledge
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: ActivePlayerPOV): Set<ConventionalAction> {
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
