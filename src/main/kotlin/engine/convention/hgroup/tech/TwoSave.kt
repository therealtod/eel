package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object TwoSave : SaveClue("2-Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: GameFromPlayerPOV): Boolean {
        val chop = getChop(teammate.hand, playerPOV)
        if (chop.index != slot.index) {
            return false
        }
        val otherPlayers = playerPOV
            .getTeammates()
            .filter { it.playerId != teammate.playerId } +
                playerPOV.getAsPlayer()

        return slot.matches{ slotIndex, card ->
            slotIndex == chop.index && run {
                val isCardRankTwo = card.rank == Rank.TWO
                val isTwoSaveLegal = canBeTwoSaved(
                    card = card,
                    otherPlayers = otherPlayers,
                    playerPOV = playerPOV,
                )
                isCardRankTwo && isTwoSaveLegal
            }
        }
    }

    override fun getGameActions(playerPOV: GameFromPlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.hand, playerPOV)
            if (teammateSlotMatchesCondition(teammate, slot = chop, playerPOV)) {
                actions.add(
                    GameActionFactory.createClueAction(
                        clueGiver = playerPOV.getOwnPlayerId(),
                        clueReceiver = teammate.playerId,
                        clueValue = Rank.TWO
                    )
                )
            }
        }
        return actions.toSet()
    }

    override fun matchesReceivedClue(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Boolean {
        val saveableTwos = getSaveableTwos(playerPOV)
        return playerPOV.getOwnHand().getSlot(focusIndex).getPossibleIdentities()
            .intersect(saveableTwos).isNotEmpty()
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Knowledge {
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

    private fun getSaveableTwos(playerPOV: GameFromPlayerPOV): Set<HanabiCard> {
        return playerPOV
            .getGameData()
            .suits
            .flatMap { it.getAllUniqueCards() }
            .filter {
                it.rank == Rank.TWO &&
                        playerPOV.getGameData().getGlobalAwayValue(it) > 0 &&
                        canBeTwoSaved(
                            card = it,
                            otherPlayers = playerPOV.getTeammates(),
                            playerPOV = playerPOV,
                        )
            }.toSet()
    }

    private fun canBeTwoSaved(
        card: HanabiCard,
        otherPlayers: Collection<Teammate>,
        playerPOV: GameFromPlayerPOV,
    ): Boolean {
        return otherPlayers.none { teammate ->
            val chop = getChop(teammate.hand, playerPOV)
            teammate.hand.countCopiesOf(card) > 0 &&
                    !chop.containsCard(card)
        }
    }
}
