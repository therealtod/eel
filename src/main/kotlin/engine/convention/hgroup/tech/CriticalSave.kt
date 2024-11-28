package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.hasChop
import eelst.ilike.engine.convention.hgroup.HGroupCommon.isGloballyKnownPlayable
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*
import eelst.ilike.game.variant.Variant

object CriticalSave
    : SaveClue() {
    override val name = "Critical Save"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
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

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val chop = getChop(receiverHand)
        val focus = getFocusedSlot(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        if (chop.index != focus.index) {
            return false
        }
        if (clueReceiver != playerPOV.playerId) {
            val teammate = playerPOV.getTeammate(clueReceiver)
            return teammateSlotMatchesCondition(
                teammate = teammate,
                slotIndex = focus.index,
                playerPOV = playerPOV
            )
        }

        val ownHand = playerPOV.ownHand
        val focusedSlot = ownHand.getSlot(focus.index)
        return focusedSlot.getPossibleIdentities()
            .any { playerPOV.globallyAvailableInfo.isCritical(it) }

    }

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
