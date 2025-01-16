package eelst.ilike.game.entity

import eelst.ilike.game.GameState
import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId

data class BaseGameState(
    override val globallyAvailableGameData: GloballyAvailableGameData,
    override val players: Map<PlayerId, Player>,
) : GameState {
    override val numberOfPlayers = players.size

    private val availableColors = globallyAvailableGameData.variant.getSuits().flatMap { it.getAssociatedColors() }
    private val availableRanks = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)

    override val defaultHandsSize = GameUtils.getHandSize(numberOfPlayers)

    override fun getPlayer(playerId: PlayerId): Player {
        return players[playerId]
            ?: throw IllegalArgumentException("No player with id: $playerId in this game")
    }

    override fun getAvailableClueValues(): Set<ClueValue> {
        return (availableColors + availableRanks).toSet()
    }

    override fun getAfter(drawAction: DrawAction): GameState {
        return this
    }

    override fun getAfter(drawAction: DrawAction, card: HanabiCard): GameState {
        return this
    }

    override fun getAfter(playAction: PlayAction): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterPlay()
        return BaseGameState(
            globallyAvailableGameData = newGloballyAvailableGameData,
            players = players,
        )
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterPlaying(playedCard)
        return BaseGameState(
            globallyAvailableGameData = newGloballyAvailableGameData,
            players = players,
        )
    }

    override fun getAfter(discardAction: DiscardAction): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterDiscard()
        return BaseGameState(
            globallyAvailableGameData = newGloballyAvailableGameData,
            players = players,
        )
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterDiscarding(discardedCard)
        return BaseGameState(
            globallyAvailableGameData = newGloballyAvailableGameData,
            players = players,
        )
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Collection<Int>): GameState {
        val newGloballyAvailableGameData = globallyAvailableGameData.getAfterClueGiven()
        return BaseGameState(
            globallyAvailableGameData = newGloballyAvailableGameData,
            players = players,
        )
    }

    val efficiency: Float = TODO()
}
