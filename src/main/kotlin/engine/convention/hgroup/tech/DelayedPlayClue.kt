package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

data object DelayedPlayClue
    : IndirectPlayClue("Delayed Play Clue") {

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        val teammateKnowsOwnSlot = engineHandlerPlayer.getHandFromPlayerPOV().getSlot(slot.index) is KnownSlot
        return !teammateKnowsOwnSlot && slot.matches { _, card ->
                    activePlayer.globallyAvailableInfo.getGlobalAwayValue(card) > 0 &&
                    connectingCardsAreKnown(card, activePlayer)
        }
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()

        activePlayer.forEachTeammate { teammate ->
            teammate
                .getSlots()
                .forEach { slot ->
                    if (teammateSlotMatchesCondition(teammate, slot, activePlayer)) {
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
        return otherTech !is SaveClue && otherTech !is DirectPlayClue
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean {
        val slot = activePlayer.getOwnHand().getSlot(focusIndex)
        return slot.getPossibleIdentities()
            .any {
                activePlayer.globallyAvailableInfo.getGlobalAwayValue(it) > 0 &&
                        connectingCardsAreKnown(it, activePlayer)
            }
    }

    private fun connectingCardsAreKnown(card: HanabiCard, activePlayer: ActivePlayer): Boolean {
        val stack = activePlayer.globallyAvailableInfo.getStackForCard(card)
        val missingCards = if (stack.isEmpty()) {
            card.getPrerequisiteCards().toSet()
        } else {
            stack.suite.getCardsBetween(stack.currentCard(), card)
        }
        return activePlayer.teamKnowsAllCards(missingCards)
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Knowledge {
        val receiverPOV = activePlayer.getTeammate(action.clueAction.clueReceiver).getPOV(activePlayer)
        val slot = receiverPOV.getOwnHand().getSlot(focusIndex)
        val possibleIdentities = slot.getPossibleIdentities()
            .filter {
                activePlayer.globallyAvailableInfo.getGlobalAwayValue(it) > 0
            }
        return KnowledgeFactory.createKnowledge(
            playerId = activePlayer.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleIdentities.toSet(),
            empathy = TODO()
        )
    }
}
