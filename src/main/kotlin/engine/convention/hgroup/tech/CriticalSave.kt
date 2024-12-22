package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object CriticalSave : SaveClue("Critical Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(
        teammate: Teammate,
        slot: Slot,
        playerPOV: GameFromPlayerPOV
    ): Boolean {
        val chop = getChop(teammate.hand, playerPOV)
        return slot.matches { index, card ->
            index == chop.index &&
            appliesTo(card, playerPOV.getGameData().variant) &&
                    card.rank != Rank.FIVE &&
                    playerPOV.getGameData().isCritical(card) &&
                    !isGloballyKnownPlayable(card, playerPOV)
        }
    }

    override fun getGameActions(playerPOV: GameFromPlayerPOV): Set<ClueAction> {
        val actions = mutableSetOf<ClueAction>()

        playerPOV.forEachTeammate { teammate ->
            if (hasChop(teammate.hand, playerPOV)) {
                val chop = getChop(teammate.hand, playerPOV)
                if (
                    teammateSlotMatchesCondition(teammate, chop, playerPOV)
                ) {
                    actions.addAll(
                        getAllCluesFocusing(
                            slot = chop,
                            teammate = teammate,
                            playerPOV = playerPOV,
                        )
                    )
                }
            }
        }
        return actions
    }

    override fun matchesReceivedClue(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Boolean {
        return playerPOV.getOwnHand().getSlot(focusIndex)
            .getPossibleIdentities()
            .any { playerPOV.getGameData().isCritical(it) }
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Knowledge {
        val receiverPOV = playerPOV.getTeammate(clueAction.clueReceiver).getPOV(playerPOV)
        val focus = receiverPOV
            .getOwnHand()
            .getSlot(focusIndex)
        val possibleFocusIdentities = focus
            .getPossibleIdentities()
            .filter {
                playerPOV.getGameData().isCritical(it)
            }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleFocusIdentities.toSet(),
            empathy = focus.getUpdatedEmpathy(clueAction.value)
        )
    }
}
