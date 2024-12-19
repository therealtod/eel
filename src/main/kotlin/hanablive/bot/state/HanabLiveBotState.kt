package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameAction
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData

abstract class HanabLiveBotState(
    protected val bot: HanabLiveBot,
    protected val commonState: CommonState,
) {
    fun setTables(tables: Collection<Table>) {
        commonState.tables.clear()
        commonState.tables.putAll(tables.associateBy { it.id })
    }

    fun putTable(table: Table) {
        commonState.tables[table.id] = table
    }

    open suspend fun joinPlayer(playerId: PlayerId) {
        throw IllegalAccessException("Cannot join a player in the current state $this")
    }

    open suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        throw IllegalAccessException("Cannot join a player in the current state $this")
    }

    open suspend fun joinTable(tableId: TableId) {
        throw IllegalAccessException("Cannot join a table in the current state $this")
    }

    open suspend fun joinTable(tableId: TableId, password: String) {
        throw IllegalAccessException("Cannot join a table in the current state $this")
    }

    open suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        throw IllegalStateException("This instruction (GameInitData) should not have been received in the current state $this")
    }

    open suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        throw IllegalStateException("This instruction (GameActionList) should not have been received in the current state $this")
    }

    open suspend fun onGameAction(gameAction: GameAction) {
        throw IllegalStateException("This instruction (GameAction) should not have been received in the current state $this")
    }
}
