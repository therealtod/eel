package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object SimplePrompt : Prompt("Simple Prompt") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean {
        return slot.matches{ _, card->
            playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) == 1 && run {
                val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
                val connectingCards = if (stack.isEmpty()) {
                    card.getPrerequisiteCards()
                } else {
                    card.suite.getCardsBetween(stack.currentCard(), card)
                }
                validatePrompt(connectingCards.toSet(), playerPOV)
            }
        }
    }


    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachVisibleTeammate { teammate ->
            teammate.getSlots().forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot, playerPOV,))
                    actions.addAll(
                        getAllCluesFocusing(
                            slotIndex = slot.index,
                            teammate = teammate,
                            playerPOV = playerPOV,
                        )
                    )
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
