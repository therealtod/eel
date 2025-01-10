package eelst.ilike.game

import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.variant.Variant


data class GameStateImpl(
    override val globallyAvailableGameData: GloballyAvailableGameData,
    override val players: Map<PlayerId, Player>
) : GameState {
    constructor(
        variant: Variant,
        playingStacks: Map<SuitId,PlayingStack>,
        trashPile: TrashPile,
        strikes: Int,
        clueTokens: Int,
        players: Map<PlayerId, Player>,
        amountOfCardsPlayed: Int,
        possibleMaxScore: Int,
    ) : this(
        globallyAvailableGameData = GloballyAvailableGameData(
            variant = variant,
            playingStacks = playingStacks,
            trashPile = trashPile,
            strikes = strikes,
            clueTokens = clueTokens,
            numberOfPlayers = players.size,
            amountOfCardsPlayed = amountOfCardsPlayed,
            possibleMaxScore = possibleMaxScore
        ),
        players = players,
    )

    override val numberOfPlayers = players.size

    private val availableColors = globallyAvailableGameData.variant.getSuits().flatMap { it.getAssociatedColors() }
    private val availableRanks = setOf(Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE)

    override val defaultHandsSize = GameUtils.getHandSize(numberOfPlayers)

    override fun getPlayer(playerId: PlayerId): Player {
        return players[playerId]
            ?: throw IllegalArgumentException("No player with id: $playerId in this game")
    }


    override fun getPlayer(playerIndex: Int): Player {
        return players.values.find { it.playerIndex == playerIndex }
            ?: throw IllegalArgumentException("Could not find any player with player index $playerIndex")
    }

    override fun getAvailableClueValues(): Set<ClueValue> {
        return (availableColors + availableRanks).toSet()
    }

    override fun getAfter(drawAction: DrawAction): GameState {
        val player = getPlayer(drawAction.playerId)
        val updatedPlayer = player.getAfterDrawing(drawAction.newSlot)
        val updatedPlayers = players
            .minus(updatedPlayer.playerId)
            .plus(Pair(updatedPlayer.playerId, updatedPlayer))
        return this.copy(
            globallyAvailableGameData = globallyAvailableGameData,
            players = updatedPlayers,
        )
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard): GameState {
        val player = getPlayer(playAction.playerId)
        val updatedPlayer = player.getAfterPlaying(playAction.slotIndex)
        val updatedPlayers = players
            .minus(updatedPlayer.playerId)
            .plus(Pair(updatedPlayer.playerId, updatedPlayer))
        return this.copy(
            globallyAvailableGameData = globallyAvailableGameData
                .getAfterPlaying(playedCard, globallyAvailableGameData.variant),
            players = updatedPlayers,
        )
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): GameState {
        val player = getPlayer(discardAction.playerId)
        val updatedPlayer = player.getAfterDiscarding(discardAction.slotIndex)
        val updatedPlayers = players
            .minus(updatedPlayer.playerId)
            .plus(Pair(updatedPlayer.playerId, updatedPlayer))
        return this.copy(
            globallyAvailableGameData = globallyAvailableGameData.getAfterDiscarding(discardedCard),
            players = updatedPlayers,
        )
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>): GameState {
        val player = getPlayer(clueAction.clueReceiver)
        val updatedPlayer = player.getAfterReceivingClue(clueAction.value, touchedSlotsIndexes)
        val updatedPlayers = players
            .minus(updatedPlayer.playerId)
            .plus(Pair(updatedPlayer.playerId, updatedPlayer))
        return this.copy(
            globallyAvailableGameData = globallyAvailableGameData.getAfterClueGiven(),
            players = updatedPlayers,
        )
    }
    val efficiency: Float = TODO()
}
