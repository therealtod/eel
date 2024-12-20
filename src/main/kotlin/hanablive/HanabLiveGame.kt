package eelst.ilike.hanablive

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.hanablive.model.dto.instruction.GameClueActionData

class HanabLiveGame(
    private val gameData: GameData,
    private val playerPOV: PlayerPOV,
    private val conventionSet: ConventionSet,
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
        return playerPOV.getPlayers()
    }

    override fun getPlayer(playerId: PlayerId): Player {
        return getPlayers()[playerId] ?: throw NoSuchElementException(
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

    override fun getAfter(playAction: PlayAction, card: HanabiCard, successful: Boolean): Game {
        val newGameData = gameData.getAfterPlaying(card)
        return HanabLiveGame(
            gameData = newGameData,
            playerPOV = playerPOV.getAfter(
                playAction = playAction,
                conventionSet = conventionSet,
            ),
            conventionSet = conventionSet,
        )
    }

    override fun getAfter(discardAction: DiscardAction): Game {
        TODO("Not yet implemented")
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>) {
        TODO("Not yet implemented")
    }
}
