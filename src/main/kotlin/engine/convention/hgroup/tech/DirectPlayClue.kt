package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object DirectPlayClue : PlayClue("Direct Play Clue") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        return !teammate.knows(slotIndex) && playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 0
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.ownHand.forEach { slot ->
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

    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue && otherTech is PlayClue
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        val ownHand = playerPOV.ownHand
        val focusedSlot = ownHand.getSlot(focusIndex)
        return focusedSlot
            .getPossibleIdentities()
            .any {
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) == 0
            }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PersonalKnowledge {
        val focusedSlot = playerPOV.ownHand.getSlot(focusIndex)
        val possibleIdentities = focusedSlot.getPossibleIdentities()
            .filter {
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) == 0
            }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.playerId,
            slotIndex = focusIndex,
            possibleIdentities = possibleIdentities.toSet()
        )
    }
}
