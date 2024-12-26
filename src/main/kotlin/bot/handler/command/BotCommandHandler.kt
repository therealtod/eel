package bot.handler.command

import eelst.ilike.bot.Bot

/**
 * Chain of responsibility for handling user commands.
 */
sealed class BotCommandHandler(private val supportedCommandType: CommandType) {
    abstract val nextHandler: BotCommandHandler

    /**
     * Handle a user command.
     */
    open suspend fun handle(
        commandType: CommandType,
        commandArgs: Collection<String>,
        requestSender: String,
        bot: Bot
    ) {
        return if (commandType == supportedCommandType) {
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
