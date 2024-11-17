package eelst.ilike.engine.convention.hgroup

import eelst.ilike.engine.InterpretedHand
import eelst.ilike.engine.PlayerPOV
import eelst.ilike.engine.Teammate
import eelst.ilike.game.Slot
import eelst.ilike.game.VisibleSlot
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

object HGroupHelper {
    fun getClueFocus(clue: Clue, hand: InterpretedHand): Slot {
        val slotTouchedByClue = hand.getSlotsTouchedBy(clue)
        require(slotTouchedByClue.isNotEmpty()) {
            "Can't determine the focus of a clue which touches no slots"
        }
        if (hasChop(hand)) {
            val chop = getChop(hand)
            return if (slotTouchedByClue.contains(chop)) {
                chop
            } else {
                slotTouchedByClue.firstOrNull { !it.isTouched() } ?: slotTouchedByClue.first()
            }
        } else {
            return slotTouchedByClue.first()
        }
    }

    fun isGloballyKnownPlayable(card: HanabiCard, playerPOV: PlayerPOV): Boolean {
        /*
        val prerequisiteCards = card.getPrerequisiteCards()
        val playedCardsForSuite = playerPOV.globallyAvailableInfo.getStackForCard(card).cards
        val teammatesKnownCards = playerPOV.teammates.flatMap { it.getKnownCards() }
        val ownKnownCards = playerPOV.getOwnKnownCards()
        return (playedCardsForSuite + teammatesKnownCards + ownKnownCards).containsAll(prerequisiteCards)

         */
        TODO()
    }

    fun isLocked(hand: InterpretedHand): Boolean {
        return hand.all { it.isTouched() }
    }

    fun hasChop(hand: InterpretedHand): Boolean {
        return !isLocked(hand)
    }

    fun getChop(hand: InterpretedHand): Slot {
        return hand.last { !it.isTouched() }
    }

    fun hasFinessePosition(hand: InterpretedHand): Boolean {
        return !isLocked(hand)
    }

    fun getFinessePosition(hand: InterpretedHand): Slot {
        return hand.first { !it.isTouched() }
    }

    fun validatePrompt(
        connectingCards: Set<HanabiCard>,
        clue: Clue,
        playerPOV: PlayerPOV
    ): Boolean {
        return isSequencePromptable(
            sequence = connectingCards,
            playerPOV = playerPOV
        )
    }

    fun isSequencePromptable(sequence: Set<HanabiCard>, playerPOV: PlayerPOV): Boolean {
        return isSequencePromptableGivenAlreadyPromptedCards(
            sequence = sequence,
            promptedCards = emptySet(),
            playerPOV = playerPOV,
        )
    }

    fun isSequencePromptableGivenAlreadyPromptedCards(
        sequence: Set<HanabiCard>,
        promptedCards: Set<HanabiCard>,
        playerPOV: PlayerPOV,
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

    fun isPromptedCorrectly(
        card: HanabiCard,
        teammate: Teammate,
        promptedSlots: Set<VisibleSlot> = emptySet(),
        playerPOV: PlayerPOV,
    ): Boolean {
        val promptedTeammateSlot = promptedSlots.firstOrNull { slot ->
            slot.isClued() &&
                    slot.fromOwnerPOV(teammate.personalInfo.getSlotInfo(slot.index))
                        .possibleIdentities.contains(card)
        } ?: return false
        val slotIdentity = teammate.hand.getSlot(promptedTeammateSlot.index).card
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

    fun wrongPromptCanBePatched(
        wrongPromptedCard: HanabiCard,
        wrongPromptedTeammate: Teammate,
        playerPOV: PlayerPOV,
    ): Boolean {
        if (playerPOV.globallyAvailableInfo.getGlobalAwayValue(wrongPromptedCard) < 0) return false
        val preRequisites = wrongPromptedCard.getPrerequisiteCards()
        val cardTeammateMap = preRequisites.associateWith {
            playerPOV.teammates.find { teammate ->
                teammate.getKnownCards().contains(it)
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

    fun hasCardOnFinessePosition(card: HanabiCard, teammate: Teammate, playerPOV: PlayerPOV): Boolean {
        if (hasFinessePosition(teammate.hand)) {
            val finessePosition = getFinessePosition(teammate.hand)
            return finessePosition == card
        } else return false
    }
}