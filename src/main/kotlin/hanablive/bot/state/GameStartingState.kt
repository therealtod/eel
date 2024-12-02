package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.client.MetadataClient
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.adapter.GloballyAvailableInfoAdapter
import eelst.ilike.hanablive.model.dto.command.GameActionData
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData
import eelst.ilike.hanablive.model.dto.instruction.GetGameInfo2

class GameStartingState(bot: HanabLiveBot) : HanabLiveBotState(bot) {
    private lateinit var globallyAvailableInfo: GloballyAvailableInfo
    private lateinit var actionList: List<GameActionData>

    override suspend fun joinTable(tableId: TableId, password: String) {
        TODO("Not yet implemented")
    }

    override suspend fun setTables(tables: Collection<Table>) {
        TODO("Not yet implemented")
    }

    override suspend fun joinTable(tableId: TableId) {
        TODO("Not yet implemented")
    }

    override suspend fun joinPlayer(playerId: PlayerId) {
        TODO("Not yet implemented")
    }

    override suspend fun putTable(table: Table) {
        TODO("Not yet implemented")
    }

    override suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        TODO("Not yet implemented")
    }

    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        val variantsMetadata = MetadataClient.getVariantsMetadata()
        val suitsMetadata = MetadataClient.getSuitsMetadata()
        val variantMetadata = variantsMetadata.find { it.name == gameInitData.options.variant }
            ?: throw IllegalStateException("Could not find metadata for game variant: ${gameInitData.options.variant}")
        globallyAvailableInfo = GloballyAvailableInfoAdapter(
            playerIds = gameInitData.playerNames.toSet(),
            variantMetadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        bot.sendHanabLiveInstruction(GetGameInfo2(gameInitData.tableID))
    }

    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
            val actionList = gameActionListData.list
        TODO()
    }
}