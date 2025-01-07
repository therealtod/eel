package eelst.ilike.hanablive

import eelst.ilike.hanablive.bot.UserCommandHandlerChainInitializer
import eelst.ilike.hanablive.instruction.handler.*

object InstructionHandlerChainInitializer {
    /**
     * Build an instruction handler chain of responsibilities and return the first handler
     */
    fun getInitializedChain(): HanabLiveInstructionHandler {
        val tableListHandler = TableListHandler(NoOpMessageHandler)
        val tableHandler = TableHandler(tableListHandler)
        val welcomeHandler = WelcomeHandler(tableHandler)
        val chainMessageHandler = ChatMessageHandler(
            userCommandHandler = UserCommandHandlerChainInitializer.getInitializedChain(),
            nextHandler = welcomeHandler
        )
        return chainMessageHandler
    }
}
