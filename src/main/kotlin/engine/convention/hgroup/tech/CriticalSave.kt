package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
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
        val chop = getChop(teammate.getVisibleHand())
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
            if (hasChop(teammate.getVisibleHand())) {
                val chop = getChop(teammate.getVisibleHand())
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
        val focusedSlot = playerPOV.getOwnSlot(focusIndex)
        return focusedSlot.getPossibleIdentities()
            .any { playerPOV.globallyAvailableInfo.isCritical(it) }

    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PersonalKnowledge {
        val focusedSlot = playerPOV.getOwnSlot(focusIndex)
        val possibleFocusIdentities = focusedSlot.getPossibleIdentities().filter {
            playerPOV.globallyAvailableInfo.isCritical(it)
        }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleFocusIdentities.toSet()
        )
    }
}
