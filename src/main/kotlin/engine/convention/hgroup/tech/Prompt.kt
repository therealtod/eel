package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.entity.card.HanabiCard

sealed class Prompt(name: String) : IndirectPlayClue(name) {
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
        val promptableTeammates = playerPOV.getTeammates()
            .filter { !it.isPOVProjection() }
            .map { it.asVisible() }

        return if (
            promptableTeammates.any { teammate ->
                isPromptedCorrectly(
                    card = nextInSequence,
                    teammate = teammate,
                    promptedSlots = teammate.getSlots(),
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
                    teammate.getSlot(slot.index)
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
                if (playerHoldingNextPrerequisite.playsBefore(playerHoldingPrerequisite, playerPOV)) {
                    return false
                }
            }
            if (cardTeammateMap[card]!!.playsBefore(wrongPromptedTeammate, playerPOV)) {
                return false
            }
        }
        return true
    }
}
