package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object SimplePrompt : Prompt("Simple Prompt") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        return slot.matches{ _, card->
            activePlayer.globallyAvailableInfo.getGlobalAwayValue(card) == 1 && run {
                val stack = activePlayer.globallyAvailableInfo.getStackForCard(card)
                val connectingCards = if (stack.isEmpty()) {
                    card.getPrerequisiteCards()
                } else {
                    card.suite.getCardsBetween(stack.currentCard(), card)
                }
                validatePrompt(connectingCards.toSet(), activePlayer)
            }
        }
    }


    override fun getGameActions(activePlayer: ActivePlayer): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        activePlayer.forEachTeammate { teammate ->
            teammate.getSlots().forEach { slot ->
                if (teammateSlotMatchesCondition(teammate, slot, activePlayer,))
                    actions.addAll(
                        getAllCluesFocusing(
                            slot = slot,
                            engineHandlerPlayer = teammate,
                            activePlayer = activePlayer,
                        )
                    )
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
