package eelst.ilike.engine.convention.hgroup.tech


import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
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

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean {
        val teammateKnowsOwnSlot = teammate.getHandFromPlayerPOV().getSlot(slot.index) is KnownSlot
        return !teammateKnowsOwnSlot && slot.matches { _, card ->
                    playerPOV.gameData.getGlobalAwayValue(card) > 0 &&
                    connectingCardsAreKnown(card, playerPOV)
        }
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()

        playerPOV.forEachTeammate { teammate ->
            teammate
                .getSlots()
                .forEach { slot ->
                    if (teammateSlotMatchesCondition(teammate, slot, playerPOV)) {
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
        return otherTech !is SaveClue && otherTech !is DirectPlayClue
    }

    override fun matchesReceivedClue(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: PlayerPOV
    ): Boolean {
        val slot = playerPOV.getOwnHand().getSlot(focusIndex)
        return slot.getPossibleIdentities()
            .any {
                playerPOV.gameData.getGlobalAwayValue(it) > 0 &&
                        connectingCardsAreKnown(it, playerPOV)
            }
    }

    private fun connectingCardsAreKnown(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        val stack = playerPOV.gameData.getStackForCard(card)
        val missingCards = if (stack.isEmpty()) {
            card.getPrerequisiteCards().toSet()
        } else {
            stack.suite.getCardsBetween(stack.currentCard(), card)
        }
        return playerPOV.teamKnowsAllCards(missingCards)
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: PlayerPOV
    ): Knowledge {
        val receiverPOV = playerPOV.getTeammate(clueAction.clueReceiver).getPOV(playerPOV)
        val slot = receiverPOV.getOwnHand().getSlot(focusIndex)
        val possibleIdentities = slot.getPossibleIdentities()
            .filter {
                playerPOV.gameData.getGlobalAwayValue(it) > 0
            }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleIdentities.toSet(),
            empathy = TODO()
        )
    }
}
