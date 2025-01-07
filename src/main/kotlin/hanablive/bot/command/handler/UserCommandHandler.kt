package eelst.ilike.hanablive.bot.command.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot

interface UserCommandHandler {
    /**
     * Tells whether the handler is compatible with the received command
     */
    fun supports(commandType: CommandType): Boolean

    /**
     * Handles the command sent by the user [requestSender]
     */
    suspend fun handle(
        commandType: CommandType,
        commandArgs: Collection<String>,
        requestSender: String,
        bot: DefaultHanabLiveBot,
    )
}
