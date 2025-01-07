package eelst.ilike.hanablive.bot

import eelst.ilike.hanablive.bot.command.handler.JoinMyTableHandler
import eelst.ilike.hanablive.bot.command.handler.NoOpBotCommandHandler
import eelst.ilike.hanablive.bot.command.handler.UserCommandHandler

object UserCommandHandlerChainInitializer {
    /**
     * Build a user command handler chain of responsibilities and return the first handler
     */
    fun getInitializedChain(): UserCommandHandler {
        val joinMyTableHandler = JoinMyTableHandler(NoOpBotCommandHandler)
        return joinMyTableHandler
    }
}
