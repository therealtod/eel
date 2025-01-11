package eelst.ilike.hanablive.bot.state


import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
import eelst.ilike.hanablive.entity.dto.instruction.GameInitData
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameAction
import hanablive.entity.dto.Table
import org.apache.logging.log4j.kotlin.Logging

abstract class HanabLiveBotState(
    protected val bot: HanabLiveBot,
    val lobbyState: LobbyState = LobbyState(),
) : Logging {
    fun setTables(tables: Collection<Table>) {
        lobbyState.tables.clear()
        lobbyState.tables.putAll(tables.associateBy { it.id })
    }

    fun putTable(table: Table) {
        lobbyState.tables[table.id] = table
    }

    open suspend fun joinPlayer(playerId: PlayerId) {
        logger.warn("Cannot join a player in the current state $this")
    }

    open suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        throw IllegalAccessException("Cannot join a player in the current state $this")
    }

    open suspend fun joinTable(tableId: TableId) {
        logger.warn("Cannot join a table in the current state $this")
    }

    open suspend fun joinTable(tableId: TableId, password: String) {
        logger.warn("Cannot join a table in the current state $this")
    }

    open suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        logger.warn(
            "This instruction (eelst.ilike.hanablive.entity.dto.instruction.GameInitData)" +
                    " should not have been received in the current state $this"
        )
    }

    open suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        logger.warn("This instruction (GameActionList) should not have been received in the current state $this")
    }

    open suspend fun onGameAction(gameAction: HanabLiveGameAction) {
        logger.warn("This instruction (GameAction) should not have been received in the current state $this")
    }

    open suspend fun leaveTable() {
        logger.info("Can't execute a leave table isntruction in the current state $this")
    }

    protected fun switchToState(newState: HanabLiveBotState) {
        bot.switchToState(newState)
    }
}
