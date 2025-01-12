package eelst.ilike.engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.slot.Slot
import game.exception.UnknownPlayerException

class TeamKnowledgeImpl(private val playersKnowledge: Map<PlayerId, PlayerKnowledge>) : TeamKnowledge {
    override fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge {
        return playersKnowledge[playerId]
            ?: throw UnknownPlayerException("Cannot find knowledge information for player with id: $playerId")
    }

    override fun getAfter(drawAction: DrawAction, newSlot: Slot): GameState {
        TODO("Not yet implemented")
    }

    override fun getAfter(
        playAction: PlayAction,
        playedCard: HanabiCard,
        isStrike: Boolean,
        conventionSet: ConventionSet
    ): GameState {
        TODO("Not yet implemented")
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        TODO("Not yet implemented")
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>): GameState {
        TODO("Not yet implemented")
    }
}
