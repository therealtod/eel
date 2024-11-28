package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*
import eelst.ilike.game.variant.Variant

data object DelayedPlayClue
    : IndirectPlayClue() {
    override val name = "Delayed Play Clue"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        return !teammate.knows(slotIndex) &&
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(card) > 0 &&
                connectingCardsAreKnown(card, playerPOV)
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()

        playerPOV.forEachTeammate { teammate ->
            teammate
                .ownHand
                .forEach { slot ->
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
        return otherTech !is SaveClue && otherTech !is DirectPlayClue
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val focus = getFocusedSlot(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        if (clueReceiver != playerPOV.playerId) {
            val teammate = playerPOV.getTeammate(clueReceiver)
            return teammateSlotMatchesCondition(
                teammate = teammate,
                slotIndex = focus.index,
                playerPOV = playerPOV,
            )
        }
        val ownHand = playerPOV.ownHand
        val focusedSlot = ownHand.getSlot(focus.index)
        return focusedSlot
            .getPossibleIdentities()
            .any {
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) > 0 &&
                        connectingCardsAreKnown(it, playerPOV)
            }

    }

    private fun connectingCardsAreKnown(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        val stack = playerPOV.globallyAvailableInfo.getStackForCard(card)
        val missingCards = if (stack.isEmpty()) {
            card.getPrerequisiteCards().toSet()
        } else {
            stack.suite.getCardsBetween(stack.currentCard(), card)
        }
        return playerPOV.teamKnowsAllCards(missingCards)
    }

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
