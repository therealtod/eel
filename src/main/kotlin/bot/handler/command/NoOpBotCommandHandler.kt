package eelst.ilike.bot.handler.request

import eelst.ilike.bot.Bot

data object NoOpBotCommandHandler : BotCommandHandler(CommandType.UNKNOWN) {
    override suspend fun doHandle(commandArgs: Collection<String>, requestSender: String, bot: Bot) {
        println("Unrecognized command")
    }

    override val nextHandler: BotCommandHandler
        get() = TODO("Not yet implemented")
}