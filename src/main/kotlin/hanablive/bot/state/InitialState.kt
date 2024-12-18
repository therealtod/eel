package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData

class InitialState(
    bot: HanabLiveBot,
    commonState: CommonState,
    username: String,
    password: String,
) : HanabLiveBotState(
    bot,
    commonState = commonState,
) {
    override suspend fun joinPlayer(playerId: PlayerId) {
        TODO("Not yet implemented")
    }

    override suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        TODO("Not yet implemented")
    }

    override suspend fun joinTable(tableId: TableId) {
        TODO("Not yet implemented")
    }

    override suspend fun joinTable(tableId: TableId, password: String) {
        TODO("Not yet implemented")
    }

    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        TODO("Not yet implemented")
    }

    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        TODO("Not yet implemented")
    }
}