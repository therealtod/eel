package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object TwoSave : SaveClue("2-Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: VisibleTeammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val chop = getChop(teammate.hand, playerPOV)
        if (chop.index != slotIndex) {
            return false
        }
        val card = teammate.getCardInSlot(slotIndex)
        val otherPlayers = playerPOV
            .getTeammates()
            .filter { it.playerId != teammate.playerId } +
                playerPOV.asTeammateOf(teammate.playerId)

        val isCardRankTwo = card.rank == Rank.TWO
        val isTwoSaveLegal = canBeTwoSaved(
            card = card,
            otherPlayers = otherPlayers,
            playerPOV = playerPOV,
        )
        return isCardRankTwo && isTwoSaveLegal
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachVisibleTeammate { teammate ->
            val chop = getChop(teammate.hand, playerPOV)
            if (teammateSlotMatchesCondition(teammate, slotIndex = chop.index, playerPOV)) {
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

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): Boolean {
        val saveableTwos = getSaveableTwos(playerPOV)
        return playerPOV.getOwnSlotPossibleIdentities(focusIndex)
            .intersect(saveableTwos).isNotEmpty()
    }

    private fun getSaveableTwos(playerPOV: PlayerPOV): Set<HanabiCard> {
        return playerPOV
            .globallyAvailableInfo
            .suits
            .flatMap { it.getAllUniqueCards() }
            .filter {
                it.rank == Rank.TWO &&
                        playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) > 0 &&
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
        playerPOV: PlayerPOV,
    ): Boolean {
        return otherPlayers.none { teammate ->
            val chop = getChop(teammate.hand, playerPOV)
            teammate.hand.countCopiesOf(card) > 0 &&
                    !chop.containsCard(card)
        }
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, playerPOV: PlayerPOV): PlayerPersonalKnowledge {
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
