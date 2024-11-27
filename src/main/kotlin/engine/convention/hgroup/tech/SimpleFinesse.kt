package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasCardOnFinessePosition
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.suite.*

object SimpleFinesse
    : Finesse(
    name = "Simple Finesse",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
    takesPrecedenceOver = emptySet(),
) {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        return playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1 &&
                playerPOV.teammates.any { otherTeammate ->
                    otherTeammate.playsBefore(teammate) &&
                            hasCardOnFinessePosition(
                                card = card.suite.cardBefore(card),
                                teammate = otherTeammate,
                            )
                }
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.hand.forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot.index, playerPOV)) {
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            slot = teammate.hand.getSlot(slot.index),
                            teammate = teammate,
                        )
                    )
                }

            }
        }
        return actions.toSet()
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.gameAction.clueReceiver
        if (clueReceiver == playerPOV.playerId) {
            return false
        }
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val focus = getFocusedSlot(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        if (playerPOV.teammates
            .filter { it.playerId != clueReceiver }
            .any {
                teammateSlotMatchesCondition(
                    teammate = it,
                    slotIndex = focus.index,
                    playerPOV = playerPOV,
                )
        }
            )
            return true

        val receivingTeammate = playerPOV.getTeammate(clueReceiver)
        val focusedCard = receivingTeammate.getCardAtSlot(focus.index)
        return playerPOV.globallyAvailableInfo.getGlobalAwayValue(focusedCard) == 1
    }

    override fun getGeneratedKnowledge(action: ObservedAction<ClueAction>, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
