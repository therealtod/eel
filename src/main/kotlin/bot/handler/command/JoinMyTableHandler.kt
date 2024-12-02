package eelst.ilike.bot.handler.request

import eelst.ilike.bot.Bot

data object JoinMyTableHandler : BotCommandHandler(CommandType.JOIN_ME) {
    override suspend fun doHandle(commandArgs: Collection<String>, requestSender: String, bot: Bot) {
        if (commandArgs.isEmpty()) {
            bot.joinPlayer(requestSender)
        } else {
            bot.joinPlayer(requestSender, commandArgs.first())
        }
    }

    override var nextHandler: BotCommandHandler
        get() = NoOpBotCommandHandler
        set(value) {}
}