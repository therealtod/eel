package eelst.ilike.hanablive.bot

import eelst.ilike.hanablive.bot.usercommand.handler.JoinMyTableHandler
import eelst.ilike.hanablive.bot.usercommand.handler.NoOpBotCommandHandler
import eelst.ilike.hanablive.bot.usercommand.handler.UserCommandHandler

object UserCommandHandlerChainInitializer {
    /**
     * Build a user command handler chain of responsibilities and return the first handler
     */
    fun getInitializedChain(): UserCommandHandler {
        val joinMyTableHandler = JoinMyTableHandler(NoOpBotCommandHandler)
        return joinMyTableHandler
    }
}
