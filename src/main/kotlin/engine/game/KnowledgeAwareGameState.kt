package eelst.ilike.engine.game

import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId

/**
 * A [GameState] as a specific [Player] would see it from their point of view
 */
class KnowledgeAwareGameState(
    override val globallyAvailableGameData: GloballyAvailableGameData,
    override val players: Map<PlayerId, Player>,
    private val teamKnowledge: TeamKnowledge,
) : GameState {
    override val defaultHandsSize: Int
        get() = TODO("Not yet implemented")
    override val numberOfPlayers: Int
        get() = TODO("Not yet implemented")

    override fun getPlayer(playerId: PlayerId): Player {
        TODO("Not yet implemented")
    }

    override fun getPlayer(playerIndex: Int): Player {
        TODO("Not yet implemented")
    }

    override fun getAvailableClueValues(): Set<ClueValue> {
        TODO("Not yet implemented")
    }

    override fun getAfter(drawAction: DrawAction): GameState {
        TODO("Not yet implemented")
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard, isStrike: Boolean): GameState {
        TODO("Not yet implemented")
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        TODO("Not yet implemented")
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>): GameState {
        TODO("Not yet implemented")
    }
}
