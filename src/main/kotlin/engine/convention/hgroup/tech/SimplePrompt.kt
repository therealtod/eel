package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
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
            playerPOV.gameData.getGlobalAwayValue(card) == 1 && run {
                val stack = playerPOV.gameData.getStackForCard(card)
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
        playerPOV.forEachTeammate { teammate ->
            teammate.getSlots().forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot, playerPOV,))
                    actions.addAll(
                        getAllCluesFocusing(
                            slot = slot,
                            teammate = teammate,
                            playerPOV = playerPOV,
                        )
                    )
            }
        }
        return actions.toSet()
    }

    override fun matchesReceivedClue(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: PlayerPOV
    ): Boolean {
        TODO("Not yet implemented")
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: PlayerPOV
    ): Knowledge {
        TODO("Not yet implemented")
    }
}
