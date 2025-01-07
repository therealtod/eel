package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import eelst.ilike.hanablive.entity.TableId
import hanablive.bot.state.TableJoinedAsPlayerState
import hanablive.entity.dto.instruction.TableJoin


class LoggedInState(
    bot: DefaultHanabLiveBot,
) : HanabLiveBotState(
    bot = bot,
) {
    override suspend fun joinPlayer(playerId: PlayerId) {
        val table = lobbyState.tables.entries.find { it.value.players.contains(playerId) }
        table?.let {
            joinTable(it.key)
        }
    }

    override suspend fun joinPlayer(playerId: PlayerId, tablePassword: String) {
        val table = lobbyState.tables.entries.find { it.value.players.contains(playerId) }
        table?.let {
            joinTable(it.key, tablePassword)
        }
    }

    override suspend fun joinTable(tableId: TableId) {
        bot.sendHanabLiveInstruction(TableJoin(tableId))
        val newState = TableJoinedAsPlayerState(bot, lobbyState, tableId)
       switchToState(newState)
    }

    override suspend fun joinTable(tableId: TableId, password: String) {
        bot.sendHanabLiveInstruction(TableJoin(tableId, password))
        val newState = TableJoinedAsPlayerState(bot, lobbyState, tableId)
        switchToState(newState)
    }
}
