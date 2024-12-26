package bot.handler.command

import eelst.ilike.bot.Bot

/**
 * Handler for the [CommandType.JOIN_ME] user command.
 * The bot will join the table that the command sender has already joined.
 */
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