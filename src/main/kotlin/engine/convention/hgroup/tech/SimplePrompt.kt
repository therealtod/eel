package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.validatePrompt
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*
import eelst.ilike.game.variant.Variant

object SimplePrompt: Prompt() {
    override val name = "Simple Prompt"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1) {
            val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
            val connectingCards = if (stack.isEmpty()) {
                card.getPrerequisiteCards()
            } else {
                card.suite.getCardsBetween(stack.currentCard(), card)
            }
            return validatePrompt(connectingCards.toSet(), playerPOV)
        }
        return false
    }


    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            teammate.hand.forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot.index, playerPOV))
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            slot = teammate.hand.getSlot(slot.index),
                            teammate = teammate,
                        )
                    )
            }
        }
        return actions.toSet()
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.clueAction.clueReceiver
        if (clueReceiver == playerPOV.playerId) {
            return false
        }
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val focus = getFocusedSlot(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        val teammate = playerPOV.getTeammate(clueReceiver)
        return teammateSlotMatchesCondition(
            teammate = teammate,
            slotIndex = focus.index,
            playerPOV = playerPOV,
        )
    }

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
