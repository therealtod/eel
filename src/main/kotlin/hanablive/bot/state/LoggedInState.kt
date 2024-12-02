package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData
import eelst.ilike.hanablive.model.dto.instruction.PasswordProtectedTableJoin
import eelst.ilike.hanablive.model.dto.instruction.TableJoin

class LoggedInState(bot: HanabLiveBot, private val tables: MutableMap<Int, Table>) : HanabLiveBotState(bot) {
    override suspend fun setTables(tables: Collection<Table>) {
        this.tables.clear()
        this.tables.putAll(tables.associateBy { it.id })
    }

    override suspend fun putTable(table: Table) {
        tables[table.id] = table
    }

    override suspend fun joinPlayer(playerId: PlayerId) {
        val table = tables.entries.find { it.value.players.contains(playerId) }
        table?.let {
            joinTable(it.key)
        }
    }

    override suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        val table = tables.entries.find { it.value.players.contains(playerId) }
        table?.let {
            joinTable(it.key, tablePassword)
        }
    }

    override suspend fun joinTable(tableId: TableId) {
        bot.sendHanabLiveInstruction(TableJoin(tableId))
        val newState = TableJoinedAsPlayerState(tableId, bot)
        bot.state = newState
    }

    override suspend fun joinTable(tableId: TableId, password: String) {
        bot.sendHanabLiveInstruction(PasswordProtectedTableJoin(tableId, password))
        val newState = TableJoinedAsPlayerState(tableId, bot)
        bot.state = newState
    }

    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        TODO("Not yet implemented")
    }

    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        TODO("Not yet implemented")
    }
}