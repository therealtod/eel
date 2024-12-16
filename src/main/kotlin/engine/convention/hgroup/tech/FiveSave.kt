package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
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

    override fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean {
        val chop = getChop(engineHandlerPlayer.hand, activePlayer)
        return slot.matches{ slotIndex, card ->
            slotIndex == chop.index && card.rank == Rank.FIVE
        }
    }

    override fun getGameActions(activePlayer: ActivePlayer): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        activePlayer.forEachTeammate { teammate ->
            val chop = getChop(teammate.hand, activePlayer)
            if (teammateSlotMatchesCondition(teammate, chop, activePlayer,)) {
                actions.add(
                    RankClueAction(
                        clueGiver = activePlayer.getOwnPlayerId(),
                        clueReceiver = teammate.playerId,
                        rank = Rank.FIVE,
                    ),
                )
            }
        }
        return actions.toSet()
    }

    override fun matchesReceivedClue(clue: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): Boolean {
        return true
    }

    override fun getGeneratedKnowledge(action: ObservedClue, focusIndex: Int, activePlayer: ActivePlayer): PlayerPersonalKnowledge {
        return KnowledgeFactory.createEmptyPersonalKnowledge()
    }
}
