package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction

object SimpleFinesse : Finesse("Simple Finesse") {
    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        return slot.matches { _, card ->
            activePlayer.globallyAvailableInfo.getGlobalAwayValue(card) == 1 &&
                    activePlayer.getTeammates().any { otherTeammate ->
                        otherTeammate.playsBefore(engineHandlerPlayer, activePlayer) &&
                                hasCardOnFinessePosition(
                                    card = card.suite.cardBefore(card),
                                    engineHandlerPlayer = otherTeammate,
                                    activePlayer = activePlayer,
                                )
                    }
        }
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        activePlayer.forEachTeammate { teammate ->
            teammate.getSlots().forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot, activePlayer,)) {
                    actions.addAll(
                        getAllCluesFocusing(
                            slot = slot,
                            engineHandlerPlayer = teammate,
                            activePlayer = activePlayer,
                        )
                    )
                }

            }
        }
        return actions.toSet()
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean {
        return false
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }
}
