package eelst.ilike.hanablive.bot

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.hanablive.bot.state.HanabLiveBotState
import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
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
     * React to the tableStart instruction
     */
    suspend fun onTableStart(tableId: TableId)

    /**
     * Send an instruction to the Hanab live server
     */
    suspend fun sendHanabLiveInstruction(instruction: HanabLiveInstruction)

    /**
     * Transition to the given [newState]
     */
    fun switchToState(newState: HanabLiveBotState)

    /**
     * @return the [VariantMetadata] for the given [variantName]
     */
    suspend fun getVariantMetadata(variantName: String): VariantMetadata

    /**
     * @return a [Map] associating each of the given [suitIds] to the corresponding [SuitMetadata]
     */
    suspend fun getSuitsMetadata(suitIds: Collection<SuitId>): Map<SuitId, SuitMetadata>

    /**
     * React to the gameActionList instruction
     */
    suspend fun onGameActionListReceived(gameActionListData: GameActionListData)

    /**
     * @return the value of the currently configured option for the [ConventionSet] to use during the game
     */
    fun getConventionSet(): ConventionSet
}
