package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object TwoSave : SaveClue("2-Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        val chop = getChop(engineHandlerPlayer.hand, activePlayer)
        if (chop.index != slot.index) {
            return false
        }
        val otherPlayers = activePlayer
            .getTeammates()
            .filter { it.playerId != engineHandlerPlayer.playerId } +
                activePlayer.getAsPlayer()

        return slot.matches{ slotIndex, card ->
            slotIndex == chop.index && run {
                val isCardRankTwo = card.rank == Rank.TWO
                val isTwoSaveLegal = canBeTwoSaved(
                    card = card,
                    otherPlayers = otherPlayers,
                    activePlayer = activePlayer,
                )
                isCardRankTwo && isTwoSaveLegal
            }
        }
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        activePlayer.forEachTeammate { teammate ->
            val chop = getChop(teammate.hand, activePlayer)
            if (teammateSlotMatchesCondition(teammate, slot = chop, activePlayer)) {
                actions.add(
                    GameActionFactory.createClueAction(
                        clueGiver = activePlayer.getOwnPlayerId(),
                        clueReceiver = teammate.playerId,
                        clueValue = Rank.TWO
                    )
                )
            }
        }
        return actions.toSet()
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean {
        val saveableTwos = getSaveableTwos(activePlayer)
        return activePlayer.getOwnHand().getSlot(focusIndex).getPossibleIdentities()
            .intersect(saveableTwos).isNotEmpty()
    }

    private fun getSaveableTwos(activePlayer: ActivePlayer): Set<HanabiCard> {
        return activePlayer
            .globallyAvailableInfo
            .suits
            .flatMap { it.getAllUniqueCards() }
            .filter {
                it.rank == Rank.TWO &&
                        activePlayer.globallyAvailableInfo.getGlobalAwayValue(it) > 0 &&
                        canBeTwoSaved(
                            card = it,
                            otherPlayers = activePlayer.getTeammates(),
                            activePlayer = activePlayer,
                        )
            }.toSet()
    }

    private fun canBeTwoSaved(
        card: HanabiCard,
        otherPlayers: Collection<EngineHandlerPlayer>,
        activePlayer: ActivePlayer,
    ): Boolean {
        return otherPlayers.none { teammate ->
            val chop = getChop(teammate.hand, activePlayer)
            teammate.hand.countCopiesOf(card) > 0 &&
                    !chop.containsCard(card)
        }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): PlayerPersonalKnowledge {
        /*
        val possibleFocusIdentities = focusedSlot.getPossibleIdentities()
            .intersect(getSaveableTwos(playerPOV))
        return KnowledgeFactory.createKnowledge(
            playerId = playerPOV.getOwnPlayerId(),
            slotIndex = focusIndex,
            possibleIdentities = possibleFocusIdentities.toSet()
        )

         */
        TODO()
    }
}
