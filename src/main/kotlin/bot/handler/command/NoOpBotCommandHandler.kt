package bot.handler.command

import bot.handler.command.BotCommandHandler
import bot.handler.command.CommandType
import eelst.ilike.bot.Bot

/**
 * Last handler of the chain, for unrecognized commands.
 */
data object NoOpBotCommandHandler : BotCommandHandler(CommandType.UNKNOWN) {
    override suspend fun doHandle(commandArgs: Collection<String>, requestSender: String, bot: Bot) {
        println("Unrecognized command")
    }

    override val nextHandler: BotCommandHandler
        get() = TODO("Not yet implemented")
}