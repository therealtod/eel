package eelst.ilike.hanablive.bot.usercommand.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot

/**
 * Handler for user commands directed to the bot
 */
abstract class BaseUserCommandHandler(private val nextHandler: UserCommandHandler) : UserCommandHandler {

    override suspend fun handle(
        commandType: CommandType,
        commandArgs: Collection<String>,
        requestSender: String,
        bot: DefaultHanabLiveBot
    ) {
        return if (supports(commandType)) {
            doHandle(commandArgs, requestSender, bot)
        } else {
            nextHandler.handle(commandType, commandArgs, requestSender, bot)
        }
    }

    protected abstract suspend fun doHandle(
        commandArgs: Collection<String>,
        requestSender: String,
        bot: DefaultHanabLiveBot
    )
}
