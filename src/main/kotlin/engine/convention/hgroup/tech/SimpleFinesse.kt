package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

object SimpleFinesse : Finesse("Simple Finesse") {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: GameFromPlayerPOV): Boolean {
        return slot.matches { _, card ->
            playerPOV.getGameData().getGlobalAwayValue(card) == 1 &&
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

    override fun getGameActions(playerPOV: GameFromPlayerPOV): Set<ClueAction> {
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

    override fun matchesReceivedClue(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Boolean {
        TODO("Not yet implemented")
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Knowledge {
        TODO("Not yet implemented")
    }
}
