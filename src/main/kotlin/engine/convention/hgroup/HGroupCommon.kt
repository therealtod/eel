package eelst.ilike.engine.convention.hgroup

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.clue.Clue
import eelst.ilike.game.entity.card.HanabiCard

object HGroupCommon {
    fun getClueFocusSlotIndex(clue: Clue, hand: VisibleHand): Int {
        val slotTouchedByClue = hand.getSlotsTouchedBy(clue)
        require(slotTouchedByClue.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        if (hasChop(hand)) {
            val chop = getChop(hand)
            return if (slotTouchedByClue.contains(chop)) {
                chop.index
            } else {
                (slotTouchedByClue.firstOrNull { !it.isTouched() } ?: slotTouchedByClue.first()).index
            }
        } else {
            return slotTouchedByClue.first().index
        }
    }

    fun isGloballyKnownPlayable(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        val prerequisiteCards = card.getPrerequisiteCards()
        val playedCardsForSuite = playerPOV.globallyAvailableInfo.getStackForCard(card).cards
        val teammatesKnownCards = playerPOV.getTeammates().flatMap { it.getOwnKnownCards() }
        val ownKnownCards = playerPOV.getOwnKnownCards()
        return (playedCardsForSuite + teammatesKnownCards + ownKnownCards).containsAll(prerequisiteCards)
    }

    private fun isLocked(hand: InterpretedHand): Boolean {
        return hand.all { it.isTouched() }
    }

    fun hasChop(hand: InterpretedHand): Boolean {
        return !isLocked(hand)
    }

    fun getChop(hand: InterpretedHand): Slot {
        return hand.last { !it.isTouched() }
    }

    private fun hasFinessePosition(hand: InterpretedHand): Boolean {
        return !isLocked(hand)
    }

    private fun getFinessePosition(hand: InterpretedHand): Slot {
        return hand.first { !it.isTouched() }
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
        promptedSlots: Set<InterpretedSlot> = emptySet(),
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
            wrongPromptedSlot = promptedTeammateSlot,
            wrongPromptedTeammate = teammate,
            playerPOV = playerPOV
        )
    }

    private fun wrongPromptCanBePatched(
        wrongPromptedSlot: InterpretedSlot,
        wrongPromptedTeammate: Teammate,
        playerPOV: PlayerPOV,
    ): Boolean {
        return false
        /*
        if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(wrongPromptedCard) < 0) return false
        val preRequisites = wrongPromptedCard.getPrerequisiteCards()
        val cardTeammateMap = preRequisites.associateWith {
            playerPOV.teammates.find { teammate ->
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

         */
    }

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: Teammate): Boolean {
        if (hasFinessePosition(teammate.hand)) {
            val finessePosition = getFinessePosition(teammate.hand)
            return teammate.hasCardInSlot(card, finessePosition.index)
        } else return false
    }
}
