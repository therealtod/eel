package eelst.ilike.hanablive.bot.command.handler

import eelst.ilike.hanablive.bot.DefaultHanabLiveBot


class JoinMyTableHandler(nextHandler: UserCommandHandler) : BaseUserCommandHandler(nextHandler) {
    override fun supports(commandType: CommandType): Boolean {
        return commandType == CommandType.JOIN_ME
    }
    override suspend fun doHandle(commandArgs: Collection<String>, requestSender: String, bot: DefaultHanabLiveBot) {
        if (commandArgs.isEmpty()) {
            bot.joinPlayer(requestSender)
        } else {
            bot.joinPlayer(requestSender, commandArgs.first())
        }
    }
}
