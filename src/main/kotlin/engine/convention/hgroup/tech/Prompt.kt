package eelst.ilike.engine.convention.hgroup.tech


import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

sealed class Prompt(name: String) : IndirectPlayClue(name) {
    fun validatePrompt(
        connectingCards: Set<HanabiCard>,
        activePlayer: ActivePlayer
    ): Boolean {
        return isSequencePromptable(
            sequence = connectingCards,
            activePlayer = activePlayer
        )
    }

    private fun isSequencePromptable(sequence: Set<HanabiCard>, activePlayer: ActivePlayer): Boolean {
        return isSequencePromptableGivenAlreadyPromptedCards(
            sequence = sequence,
            promptedCards = emptySet(),
            activePlayer = activePlayer,
        )
    }

    private fun isSequencePromptableGivenAlreadyPromptedCards(
        sequence: Set<HanabiCard>,
        promptedCards: Set<HanabiCard>,
        activePlayer: ActivePlayer,
    ): Boolean {
        if (sequence.isEmpty()) return true
        val nextInSequence = sequence.first()
        val promptableTeammates = activePlayer.getTeammates()

        return if (
            promptableTeammates.any { teammate ->
                isPromptedCorrectly(
                    card = nextInSequence,
                    engineHandlerPlayer = teammate,
                    promptedSlots = teammate.getSlots(),
                    activePlayer = activePlayer,
                )
            }
        ) {
            isSequencePromptableGivenAlreadyPromptedCards(
                sequence = sequence.minus(nextInSequence),
                promptedCards = promptedCards + nextInSequence,
                activePlayer = activePlayer,
            )
        } else {
            false
        }
    }

    private fun isPromptedCorrectly(
        card: HanabiCard,
        engineHandlerPlayer: EngineHandlerPlayer,
        promptedSlots: Set<Slot> = emptySet(),
        activePlayer: ActivePlayer,
    ): Boolean {
        val promptedTeammateSlot = promptedSlots.firstOrNull { slot ->
            slot.isTouched() &&
                    engineHandlerPlayer.getPOV(activePlayer)
                        .getOwnHand()
                        .getSlot(slot.index)
                        .getPossibleIdentities()
                        .contains(card)
        } ?: return false
        return promptedTeammateSlot.matches { _, identity -> identity == card } ||
                (activePlayer.globallyAvailableInfo.isImmediatelyPlayable(card)
                        && isPromptedCorrectly(
                    card = card,
                    engineHandlerPlayer = engineHandlerPlayer,
                    promptedSlots = promptedSlots.filter { it.index < promptedTeammateSlot.index }.toSet(),
                    activePlayer = activePlayer
                )
                        ) || wrongPromptCanBePatched(
            wrongPromptedSlot = promptedTeammateSlot,
            wrongPromptedEngineHandlerPlayer = engineHandlerPlayer,
            activePlayer = activePlayer
        )
    }

    private fun wrongPromptCanBePatched(
        wrongPromptedSlot: Slot,
        wrongPromptedEngineHandlerPlayer: EngineHandlerPlayer,
        activePlayer: ActivePlayer,
    ): Boolean {
        if (wrongPromptedSlot !is VisibleSlot) {
            return false
        }
        val wrongPromptedCard = wrongPromptedSlot.knownIdentity
        if (activePlayer.globallyAvailableInfo.getGlobalAwayValue(wrongPromptedCard) < 0) return false
        val preRequisites = wrongPromptedCard.getPrerequisiteCards()
        val cardTeammateMap = preRequisites.associateWith {
            activePlayer.getTeammates().find { teammate ->
                teammate.getPOV(activePlayer).getOwnKnownCards().contains(it)
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
                if (playerHoldingNextPrerequisite.playsBefore(playerHoldingPrerequisite, activePlayer)) {
                    return false
                }
            }
            if (cardTeammateMap[card]!!.playsBefore(wrongPromptedEngineHandlerPlayer, activePlayer)) {
                return false
            }
        }
        return true
    }
}
