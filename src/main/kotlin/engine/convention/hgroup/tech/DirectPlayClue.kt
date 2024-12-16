package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DirectPlayClue : PlayClue("Direct Play Clue") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        val slotFromTeammatePOV = engineHandlerPlayer.getHandFromPlayerPOV().getSlot(slot.index)
        val teammateKnowsOwnSlot = slotFromTeammatePOV is KnownSlot
        return !teammateKnowsOwnSlot && slot.matches{ _, card ->
            activePlayer.globallyAvailableInfo.getGlobalAwayValue(card) == 0
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

    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech is PlayClue
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean {
        val slot = activePlayer.getOwnHand().getSlot(focusIndex)
        return slot.getPossibleIdentities()
            .any {
                activePlayer.globallyAvailableInfo.getGlobalAwayValue(it) == 0
            }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): PlayerPersonalKnowledge {
        /*
        val focusedSlot = playerPOV.getSlot(focusIndex)
        val possibleIdentities = playerPOV.getPossibleSlotIdentities(focusedSlot)
            .filter {
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) == 0
            }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleIdentities.toSet()
        )

         */
        TODO()
    }
}
