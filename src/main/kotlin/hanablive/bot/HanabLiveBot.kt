package eelst.ilike.hanablive.bot

import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.hanablive.bot.state.HanabLiveBotState
import hanablive.entity.dto.instruction.HanabLiveInstruction

interface HanabLiveBot {
    /**
     * Join the table at which the player with the given [playerId] is sitting
     */
    suspend fun joinPlayer(playerId: PlayerId)

    /**
     * Join the password protected table at which the player with the given [playerId] is sitting,
     * using the given [tablePassword]
     */
    suspend fun joinPlayer(playerId: PlayerId, tablePassword: String)

    /**
     * Leave the table at which the bot is currently sitting
     */
    suspend fun leaveTable()

    /**
     * Send an instruction to the Hanab live server
     */
    suspend fun sendHanabLiveInstruction(instruction: HanabLiveInstruction)

    fun switchToState(newState: HanabLiveBotState)
}