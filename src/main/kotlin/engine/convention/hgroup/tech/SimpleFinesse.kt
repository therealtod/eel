package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

object SimpleFinesse : Finesse("Simple Finesse") {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean {
        return slot.matches { _, card ->
            playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1 &&
                    playerPOV.getTeammates().any { otherTeammate ->
                        otherTeammate.playsBefore(teammate, playerPOV) &&
                                hasCardOnFinessePosition(
                                    card = card.suite.cardBefore(card),
                                    teammate = otherTeammate,
                                    playerPOV = playerPOV,
                                )
                    }
        }
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.getSlots().forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot, playerPOV,)) {
                    actions.addAll(
                        getAllCluesFocusing(
                            slot = slot,
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
