package eelst.ilike.engine.convention.hgroup

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

object HGroupCommon {


    fun isGloballyKnownPlayable(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        val prerequisiteCards = card.getPrerequisiteCards()
        val playedCardsForSuite = playerPOV.globallyAvailableInfo.getStackForCard(card).cards
        val teammatesKnownCards = playerPOV.getTeammates().flatMap { playerPOV.getCardsKnownByTeammate(it.playerId) }
        val ownKnownCards = playerPOV.getOwnKnownCards()
        return (playedCardsForSuite + teammatesKnownCards + ownKnownCards).containsAll(prerequisiteCards)
    }

    fun hasChop(hand: InterpretedHand): Boolean {
        return hand.any { !it.isTouched() }
    }

    fun getChop(hand: InterpretedHand): InterpretedSlot {
        return hand.last { !it.isTouched() }
    }

    private fun hasFinessePosition(hand: InterpretedHand): Boolean {
        return hand.any { !it.isTouched() }
    }

    private fun getFinessePosition(hand: InterpretedHand): Slot {
        return hand.first { !it.isTouched() }
    }

    fun getFocusedSlot(
        hand: VisibleHand,
        clueValue: ClueValue,
    ): InterpretedSlot {
        val touchedSlotsIndexes = hand.getSlotsTouchedBy(clueValue)
        return getFocusedSlot(hand, touchedSlotsIndexes.map { it.index }.toSet())
    }

    fun getFocusedSlot(
        hand: InterpretedHand,
        touchedSlotsIndexes: Set<Int>,
    ): InterpretedSlot {
        require(touchedSlotsIndexes.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        val touchedSlots = touchedSlotsIndexes.map { hand.getSlot(it) }
        if (hasChop(hand)) {
            val chop = getChop(hand)
            return if (touchedSlotsIndexes.contains(chop.index)) {
                chop
            } else {
                (touchedSlots.firstOrNull { !it.isTouched() } ?: touchedSlots.first())
            }
        } else {
            return touchedSlots.first()
        }
    }

    fun validatePrompt(
        connectingCards: Set<HanabiCard>,
        playerPOV: PlayerPOV
    ): Boolean {
        return isSequencePromptable(
            sequence = connectingCards,
            playerPOV = playerPOV
        )
    }

    private fun isSequencePromptable(sequence: Set<HanabiCard>, playerPOV: PlayerPOV): Boolean {
        return isSequencePromptableGivenAlreadyPromptedCards(
            sequence = sequence,
            promptedCards = emptySet(),
            playerPOV = playerPOV,
        )
    }

    private fun isSequencePromptableGivenAlreadyPromptedCards(
        sequence: Set<HanabiCard>,
        promptedCards: Set<HanabiCard>,
        playerPOV: PlayerPOV,
    ): Boolean {
        if (sequence.isEmpty()) return true
        val nextInSequence = sequence.first()
        return if (
            playerPOV.getTeammates().any { teammate ->
                isPromptedCorrectly(
                    card = nextInSequence,
                    teammate = teammate,
                    promptedSlots = teammate.hand.getSlots(),
                    playerPOV = playerPOV,
                )
            }
        ) {
            isSequencePromptableGivenAlreadyPromptedCards(
                sequence = sequence.minus(nextInSequence),
                promptedCards = promptedCards + nextInSequence,
                playerPOV = playerPOV,
            )
        } else {
            false
        }
    }

    private fun isPromptedCorrectly(
        card: HanabiCard,
        teammate: Teammate,
        promptedSlots: Set<VisibleSlot> = emptySet(),
        playerPOV: PlayerPOV,
    ): Boolean {
        val promptedTeammateSlot = promptedSlots.firstOrNull { slot ->
            slot.isClued() &&
                    teammate.getSlotFromTeammatePOV(slot.index)
                        .getPossibleIdentities()
                        .contains(card)
        } ?: return false
        return (promptedTeammateSlot.contains(card)) ||
                (playerPOV.globallyAvailableInfo.isImmediatelyPlayable(card)
                        && isPromptedCorrectly(
                    card = card,
                    teammate = teammate,
                    promptedSlots = promptedSlots.filter { it.index < promptedTeammateSlot.index }.toSet(),
                    playerPOV = playerPOV
                )
                        ) || wrongPromptCanBePatched(
            wrongPromptedCard = promptedTeammateSlot.card,
            wrongPromptedTeammate = teammate,
            playerPOV = playerPOV
        )
    }

    private fun wrongPromptCanBePatched(
        wrongPromptedCard: HanabiCard,
        wrongPromptedTeammate: Teammate,
        playerPOV: PlayerPOV,
    ): Boolean {
        if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(wrongPromptedCard) < 0) return false
        val preRequisites = wrongPromptedCard.getPrerequisiteCards()
        val cardTeammateMap = preRequisites.associateWith {
            playerPOV.getTeammates().find { teammate ->
                teammate.getOwnKnownCards().contains(it)
            }
        }
        if (cardTeammateMap.entries.any { it.value == null }) {
            return false
        }
        val suite = wrongPromptedCard.suite
        val sortedKeys = cardTeammateMap.keys.sortedBy { suite.getPlayingOrder(it) }
        sortedKeys.forEachIndexed { index, card ->
            if (index > 0) {
                val playerHoldingNextPrerequisite = cardTeammateMap[card]!!
                val playerHoldingPrerequisite = cardTeammateMap[sortedKeys.elementAt(index - 1)]!!
                if (playerHoldingNextPrerequisite.playsBefore(playerHoldingPrerequisite)) {
                    return false
                }
            }
            if (cardTeammateMap[card]!!.playsBefore(wrongPromptedTeammate)) {
                return false
            }
        }
        return true
    }

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: Teammate): Boolean {
        if (hasFinessePosition(teammate.hand)) {
            val finessePosition = getFinessePosition(teammate.hand)
            return teammate.hasCardInSlot(card, finessePosition.index)
        } else return false
    }
}
