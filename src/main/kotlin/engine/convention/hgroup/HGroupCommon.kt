package eelst.ilike.engine.convention.hgroup

import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.ActivePlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

object HGroupCommon {
    fun getClueFocusSlotIndex(clue: Clue, hand: InterpretedHand): Int {
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

    fun isGloballyKnownPlayable(card: HanabiCard, playerPOV: ActivePlayerPOV): Boolean {
        val prerequisiteCards = card.getPrerequisiteCards()
        val playedCardsForSuite = playerPOV.globallyAvailableInfo.getStackForCard(card).cards
        val teammatesKnownCards = playerPOV.teammates.flatMap { it.getOwnKnownCards() }
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
        playerPOV: ActivePlayerPOV
    ): Boolean {
        return isSequencePromptable(
            sequence = connectingCards,
            playerPOV = playerPOV
        )
    }

    private fun isSequencePromptable(sequence: Set<HanabiCard>, playerPOV: ActivePlayerPOV): Boolean {
        return isSequencePromptableGivenAlreadyPromptedCards(
            sequence = sequence,
            promptedCards = emptySet(),
            playerPOV = playerPOV,
        )
    }

    private fun isSequencePromptableGivenAlreadyPromptedCards(
        sequence: Set<HanabiCard>,
        promptedCards: Set<HanabiCard>,
        playerPOV: ActivePlayerPOV,
    ): Boolean {
        if (sequence.isEmpty()) return true
        val nextInSequence = sequence.first()
        return if (
            playerPOV.teammates.any { teammate ->
                isPromptedCorrectly(
                    card = nextInSequence,
                    teammate = teammate,
                    promptedSlots = teammate.hand.slots,
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
        playerPOV: ActivePlayerPOV,
    ): Boolean {
        val promptedTeammateSlot = promptedSlots.firstOrNull { slot ->
            slot.isClued() &&
                    teammate.getSlotFromPlayerPOV(slot.index)
                        .getPossibleIdentities()
                        .contains(card)
        } ?: return false
        val slotIdentity = teammate.getSlot(promptedTeammateSlot.index).card
        return (slotIdentity == card) ||
                (playerPOV.globallyAvailableInfo.isImmediatelyPlayable(card)
                        && isPromptedCorrectly(
                    card = card,
                    teammate = teammate,
                    promptedSlots = promptedSlots.filter { it.index < promptedTeammateSlot.index }.toSet(),
                    playerPOV = playerPOV
                )
                        ) || wrongPromptCanBePatched(
            wrongPromptedCard = slotIdentity,
            wrongPromptedTeammate = teammate,
            playerPOV = playerPOV
        )
    }

    private fun wrongPromptCanBePatched(
        wrongPromptedCard: HanabiCard,
        wrongPromptedTeammate: Teammate,
        playerPOV: ActivePlayerPOV,
    ): Boolean {
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
                if (cardTeammateMap[card]!!.seatsGap <= cardTeammateMap[sortedKeys.elementAt(index - 1)]!!.seatsGap) {
                    return false
                }
            }
            if (cardTeammateMap[card]!!.seatsGap >= wrongPromptedTeammate.seatsGap) {
                return false
            }
        }
        return true
    }

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: Teammate): Boolean {
        if (hasFinessePosition(teammate.hand)) {
            val finessePosition = getFinessePosition(teammate.hand)
            return teammate.getSlot(finessePosition.index).card == card
        } else return false
    }


    fun getChop(teammate: Teammate): VisibleSlot {
        require(hasChop(teammate.hand)) {
            "Cannot find a chop in ${teammate.playerId}'s hand"
        }
        val slot = getChop(teammate.hand)
        return teammate.getSlot(slot.index)
    }
}