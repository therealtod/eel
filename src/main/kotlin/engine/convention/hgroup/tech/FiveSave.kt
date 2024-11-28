package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.RankClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*
import eelst.ilike.game.variant.Variant

object FiveSave : SaveClue() {
    override val name = "5-Save"

    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        return card.rank == Rank.FIVE
    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.ownHand)
            if (teammateSlotMatchesCondition(teammate, chop.index, playerPOV)) {
                actions.add(
                    RankClueAction(
                        clueGiver = playerPOV.playerId,
                        clueReceiver = teammate.playerId,
                        rank = Rank.FIVE,
                    ),
                )
            }
        }
        return actions.toSet()
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.clueAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val chop = getChop(receiverHand)
        val touchedSlotIndexes = action.slotsTouched
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
                playerPOV = playerPOV,
            )
        }
        return true
    }

    override fun getGeneratedKnowledge(action: ObservedClue, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
