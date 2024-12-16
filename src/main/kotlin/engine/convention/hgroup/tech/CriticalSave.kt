package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
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
        engineHandlerPlayer: EngineHandlerPlayer,
        slot: Slot,
        activePlayer: ActivePlayer
    ): Boolean {
        val chop = getChop(engineHandlerPlayer.hand, activePlayer)
        return slot.matches { index, card ->
            index == chop.index &&
            appliesTo(card, activePlayer.globallyAvailableInfo.variant) &&
                    card.rank != Rank.FIVE &&
                    activePlayer.globallyAvailableInfo.isCritical(card) &&
                    !isGloballyKnownPlayable(card, activePlayer)
        }
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<ClueAction> {
        val actions = mutableSetOf<ClueAction>()

        activePlayer.forEachTeammate { teammate ->
            if (hasChop(teammate.hand, activePlayer)) {
                val chop = getChop(teammate.hand, activePlayer)
                if (
                    teammateSlotMatchesCondition(teammate, chop, activePlayer)
                ) {
                    actions.addAll(
                        getAllCluesFocusing(
                            slot = chop,
                            engineHandlerPlayer = teammate,
                            activePlayer = activePlayer,
                        )
                    )
                }
            }
        }
        return actions
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean {
        return activePlayer.getOwnHand().getSlot(focusIndex)
            .getPossibleIdentities()
            .any { activePlayer.globallyAvailableInfo.isCritical(it) }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Knowledge {
        val receiverPOV = activePlayer.getTeammate(action.clueAction.clueReceiver).getPOV(activePlayer)
        val focus = receiverPOV
            .getOwnHand()
            .getSlot(focusIndex)
        val possibleFocusIdentities = focus
            .getPossibleIdentities()
            .filter {
                activePlayer.globallyAvailableInfo.isCritical(it)
        }
        return KnowledgeFactory.createKnowledge(
            playerId = activePlayer.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleFocusIdentities.toSet(),
            empathy = focus.getUpdatedEmpathy(action.clueAction.value)
        )
    }
}
