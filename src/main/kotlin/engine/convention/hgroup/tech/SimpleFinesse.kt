package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.action.ClueAction

object SimpleFinesse : Finesse("Simple Finesse") {
    override fun teammateSlotMatchesCondition(teammate: VisibleTeammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardInSlot(slotIndex)
        return playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1 &&
                playerPOV.getVisibleTeammates().any { otherTeammate ->
                    otherTeammate.playsBefore(teammate, playerPOV) &&
                            hasCardOnFinessePosition(
                                card = card.suite.cardBefore(card),
                                teammate = otherTeammate,
                                playerPOV = playerPOV,
                            )
                }
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachVisibleTeammate { teammate ->
            teammate.getSlots().forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot.index, playerPOV,)) {
                    actions.addAll(
                        getAllCluesFocusing(
                            slotIndex = slot.index,
                            teammate = teammate,
                            playerPOV = playerPOV,
                        )
                    )
                }

            }
        }
        return actions.toSet()
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }
}
