package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object CriticalSave : SaveClue("Critical Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val chop = getChop(teammate.hand)
        if (chop.index != slotIndex) {
            return false
        }
        val card = teammate.getCardAtSlot(slotIndex)
        return appliesTo(card, playerPOV.globallyAvailableInfo.variant) &&
                card.rank != Rank.FIVE &&
                playerPOV.globallyAvailableInfo.isCritical(card) &&
                !isGloballyKnownPlayable(card, playerPOV)
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableSetOf<ClueAction>()

        playerPOV.forEachTeammate { teammate ->
            if (hasChop(teammate.hand)) {
                val chop = getChop(teammate.hand)
                val teammateSlot = teammate.hand.getSlot(chop.index)
                if (
                    teammateSlotMatchesCondition(teammate, chop.index, playerPOV)
                ) {
                    actions.addAll(
                        getAllFocusingClues(
                            playerPOV = playerPOV,
                            slot = teammateSlot,
                            teammate = teammate,
                        )
                    )
                }
            }
        }
        return actions
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        val ownHand = playerPOV.ownHand
        val focusedSlot = ownHand.getSlot(focusIndex)
        return focusedSlot.getPossibleIdentities()
            .any { playerPOV.globallyAvailableInfo.isCritical(it) }

    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PersonalKnowledge {
        val focusedSlot = playerPOV.ownHand.getSlot(focusIndex)
        val possibleFocusIdentities = focusedSlot.getPossibleIdentities().filter {
            playerPOV.globallyAvailableInfo.isCritical(it)
        }
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.playerId,
            slotIndex = focusIndex,
            possibleIdentities = possibleFocusIdentities.toSet()
        )
    }
}
