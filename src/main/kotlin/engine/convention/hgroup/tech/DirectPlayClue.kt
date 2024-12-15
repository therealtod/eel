package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DirectPlayClue : PlayClue("Direct Play Clue") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean {
        val slotFromTeammatePOV = teammate.getHandFromPlayerPOV().getSlot(slot.index)
        val teammateKnowsOwnSlot = slotFromTeammatePOV is KnownSlot
        return !teammateKnowsOwnSlot && slot.matches{ _, card ->
            playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0
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

    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech is PlayClue
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        val slot = playerPOV.getOwnHand().getSlot(focusIndex)
        return slot.getPossibleIdentities()
            .any {
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) == 0
            }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
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
