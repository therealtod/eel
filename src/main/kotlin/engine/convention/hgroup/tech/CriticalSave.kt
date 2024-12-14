package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Hand
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object CriticalSave : SaveClue("Critical Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(
        teammate: VisibleTeammate,
        slotIndex: Int,
        playerPOV: PlayerPOV
    ): Boolean {
        val chop = getChop(teammate.hand, playerPOV)
        if (chop.index != slotIndex) {
            return false
        }
        val card = teammate.getCardInSlot(slotIndex)
        return appliesTo(card, playerPOV.globallyAvailableInfo.variant) &&
                card.rank != Rank.FIVE &&
                playerPOV.globallyAvailableInfo.isCritical(card) &&
                !isGloballyKnownPlayable(card, playerPOV)
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableSetOf<ClueAction>()

        playerPOV.forEachVisibleTeammate { teammate ->
            if (hasChop(teammate.hand, playerPOV)) {
                val chop = getChop(teammate.hand, playerPOV)
                if (
                    teammateSlotMatchesCondition(teammate, chop.index, playerPOV)
                ) {
                    actions.addAll(
                        getAllCluesFocusing(
                            slotIndex = chop.index,
                            teammate = teammate,
                            playerPOV = playerPOV,
                        )
                    )
                }
            }
        }
        return actions
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        return playerPOV.getOwnSlotPossibleIdentities(focusIndex)
            .any { playerPOV.globallyAvailableInfo.isCritical(it) }

    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Knowledge {
        val receiverPOV = playerPOV.getTeammate(action.clueAction.clueReceiver).getPOV(playerPOV)
        val possibleFocusIdentities = receiverPOV
            .getOwnSlotPossibleIdentities(focusIndex).filter {
            playerPOV.globallyAvailableInfo.isCritical(it)
        }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleFocusIdentities.toSet(),
            empathy = receiverPOV.getOwnSlotEmpathy(focusIndex)
        )
    }
}
