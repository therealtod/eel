package eelst.ilike.hanablive

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.hanablive.model.dto.instruction.GameClueActionData

class HanabLiveGame(
    private val gameData: GameData,
    private val players: Map<PlayerId, Player>
): Game {
    private val idToColorMap = gameData.variant.getCluableColors().size.downTo(1)
        .associateWith { gameData.variant.getCluableColors().elementAt(it - 1) }

    private val idToRankMap = gameData.variant.getCluableRanks().size.downTo(1)
        .associateWith { gameData.variant.getCluableRanks().elementAt(it - 1) }

    fun getClueValue(clue: GameClueActionData.Clue): ClueValue {
        return when (clue.type) {
            HanabLiveConstants.COLOR_CLUE_TYPE -> idToColorMap[clue.value]!!
                HanabLiveConstants.RANK_CLUE_TYPE -> idToRankMap[clue.value]!!
            else -> throw UnsupportedOperationException("Hanab Live clue type ${clue.type} is unsupported")
        }
    }

    override fun getGameData(): GameData {
        return gameData
    }

    override fun getPlayers(): Map<PlayerId, Player> {
        return players
    }

    override fun getPlayer(playerId: PlayerId): Player {
        return players[playerId] ?: throw NoSuchElementException(
            "No player with id $playerId in this game"
        )
    }

    override fun getPlayerMetadata(playerId: PlayerId): PlayerMetadata {
        return gameData.getPlayerMetadata(playerId)
    }

    override fun getPlayerMetadata(playerIndex: Int): PlayerMetadata {
        return gameData.getPlayerMetadata(playerIndex)
    }

    fun getPlayerSlots(playerId: PlayerId, hanabLiveSlotIds: Collection<Int>): Set<Int> {
        TODO()
    }

    fun getPlayerSlot(playerId: PlayerId, hanabLiveSlotId: Int): Int {
        TODO()
    }

    override fun getAfter(observedAction: ObservedAction): Game {
        TODO("Not yet implemented")
    }
}
