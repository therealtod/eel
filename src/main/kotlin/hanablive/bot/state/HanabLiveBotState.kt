package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData

abstract class HanabLiveBotState(protected val bot: HanabLiveBot) {
    abstract suspend fun setTables(tables: Collection<Table>)

    abstract suspend fun putTable(table: Table)

    abstract suspend fun joinPlayer(playerId: PlayerId)

    abstract suspend fun joinPlayer(playerId: PlayerId, tablePassword: String)

    abstract suspend fun joinTable(tableId: TableId)

    abstract suspend fun joinTable(tableId: TableId, password: String)

    abstract suspend fun onGameInitDataReceived(gameInitData: GameInitData)

    abstract suspend fun onGameActionListReceived(gameActionListData: GameActionListData)
}
