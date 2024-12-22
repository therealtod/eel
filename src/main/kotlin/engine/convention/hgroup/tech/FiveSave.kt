package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.RankClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

object FiveSave : SaveClue("5-Save") {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: GameFromPlayerPOV): Boolean {
        val chop = getChop(teammate.hand, playerPOV)
        return slot.matches{ slotIndex, card ->
            slotIndex == chop.index && card.rank == Rank.FIVE
        }
    }

    override fun getGameActions(playerPOV: GameFromPlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.hand, playerPOV)
            if (teammateSlotMatchesCondition(teammate, chop, playerPOV,)) {
                actions.add(
                    RankClueAction(
                        clueGiver = playerPOV.getOwnPlayerId(),
                        clueReceiver = teammate.playerId,
                        rank = Rank.FIVE,
                    ),
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
        return true
    }

    override fun getGeneratedKnowledge(
        clueAction: ClueAction,
        touchedSlotsIndexes: Set<Int>,
        focusIndex: Int,
        playerPOV: GameFromPlayerPOV
    ): Knowledge {
        TODO("Not yet implemented")
    }
}
