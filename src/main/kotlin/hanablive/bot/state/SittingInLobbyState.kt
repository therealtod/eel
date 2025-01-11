package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.hanablive.HanabLiveConstants
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.BotMessageTemplate
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.instruction.ChatPM
import hanablive.bot.state.TableJoinedAsPlayerState
import hanablive.entity.dto.instruction.TableJoin


class SittingInLobbyState(
    bot: HanabLiveBot,
    lobbyState: LobbyState = LobbyState()
) : HanabLiveBotState(
    bot = bot,
    lobbyState = lobbyState,
) {
    override suspend fun joinPlayer(playerId: PlayerId) {
        val table = lobbyState.tables.entries.find { it.value.players.contains(playerId) }
        if (table == null) {
            logger.info("Could not determine the table to join player $playerId")
            val messageForUser = BotMessageTemplate.CANNOT_FIND_TABLE_TO_JOIN_PLAYER.template
            val instruction = ChatPM(
                message = messageForUser,
                recipient = playerId,
                room = HanabLiveConstants.LOBBY_ROOM_NAME
            )
            bot.sendHanabLiveInstruction(instruction)
        } else {
            joinTable(table.key)
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
        val newState = TableJoinedAsPlayerState(bot, lobbyState, tableId)
        switchToState(newState)
        bot.sendHanabLiveInstruction(TableJoin(tableId, password))
    }
}
