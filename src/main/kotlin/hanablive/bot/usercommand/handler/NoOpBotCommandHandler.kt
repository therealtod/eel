package eelst.ilike.hanablive.bot.usercommand.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot
import org.apache.logging.log4j.kotlin.Logging


object NoOpBotCommandHandler : UserCommandHandler, Logging {
    override fun supports(commandType: CommandType): Boolean {
        return true
    }


    override suspend fun handle(
        commandType: CommandType,
        commandArgs: Collection<String>,
        requestSender: String,
        bot: DefaultHanabLiveBot
    ) {
        logger.info("Received an unrecognized command from $requestSender. Args: $commandArgs")
    }
}
