package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DirectPlayClue : PlayClue("Direct Play Clue") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: VisibleTeammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardInSlot(slotIndex)
        return !teammate.knowsIdentityOfOwnSlot(slotIndex) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0
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

    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech is PlayClue
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        return playerPOV.getPossibleSlotIdentities(focusIndex, playerPOV.getOwnPlayerId())
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
