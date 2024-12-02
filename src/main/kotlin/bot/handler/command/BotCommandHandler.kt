package eelst.ilike.bot.handler.request

import eelst.ilike.bot.Bot

sealed class BotCommandHandler(private val supportedCommandType: CommandType) {
    abstract val nextHandler: BotCommandHandler

    open suspend fun handle(
        commandType: CommandType,
        commandArgs: Collection<String>,
        requestSender: String,
        bot: Bot
    ) {
        return if(commandType == supportedCommandType) {
            doHandle(commandArgs, requestSender, bot)
        } else {
            nextHandler.handle(commandType, commandArgs, requestSender, bot)
        }
    }

    protected abstract suspend fun doHandle(
        commandArgs: Collection<String>,
        requestSender: String,
        bot: Bot
    )
}