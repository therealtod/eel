package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.PlayerPOVImpl
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.*

object CriticalSave
    : SaveClue(
    name = "Critical Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun getActions(playerPOV: PlayerPOV): Set<ConventionalAction> {
        return recognize(playerPOV)
            .map {
                ConventionalAction(
                    action = it,
                    tech = this,
                )
            }.toSet()
    }

    fun recognize(playerPOV: PlayerPOV): Set<GameAction> {
        val actions = mutableSetOf<GameAction>()

        playerPOV.forEachTeammate { teammate ->
            if (hasChop(teammate.ownHand)) {
                val chop = getChop(teammate.ownHand)
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
                            hand = teammate.ownHand,
                        ).map {
                            TODO()
                        }
                    )
                }
            }
        }
        return actions
    }
}
