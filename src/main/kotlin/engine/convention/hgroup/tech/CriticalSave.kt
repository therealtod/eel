package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupHelper.getChop
import eelst.ilike.engine.convention.hgroup.HGroupHelper.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupHelper.isGloballyKnownPlayable
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.NoVarBlue
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarPurple
import eelst.ilike.game.entity.suite.NoVarRed
import eelst.ilike.game.entity.suite.NoVarYellow

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(NoVarRed, NoVarYellow, NoVarGreen, NoVarBlue, NoVarPurple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        val actions = mutableListOf<ConventionalAction>()

        playerPOV.teammates.forEach { teammate ->
            if (hasChop(teammate.hand)) {
                val chop = getChop(teammate.hand)
                val card = chop.getCard()
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
