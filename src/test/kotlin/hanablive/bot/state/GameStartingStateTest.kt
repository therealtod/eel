package hanablive.bot.state

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.bot.state.CommonState
import eelst.ilike.hanablive.bot.state.GameInitDataReceivedState
import eelst.ilike.hanablive.bot.state.GameStartingState
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.instruction.GetGameInfo2
import eelst.ilike.utils.Utils
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class GameStartingStateTest {
    private val mapper = Utils.jsonObjectMapper

    private val bot = mockk<HanabLiveBot>(relaxed = true)

    private val state = GameStartingState(bot, CommonState())

    @Test
    fun `Should send the correct instruction back via the bot and change the state to GameInitDataReceived`() {
        val gameInitDataPayloadAsString =
            Utils.getResourceFileContentAsString("hanablive/gamestarting/game_init_payload.json")
        val gameInitData: GameInitData = mapper.readValue(gameInitDataPayloadAsString)

        runBlocking { state.onGameInitDataReceived(gameInitData) }

        coVerify(exactly = 1) {
            bot.sendHanabLiveInstruction(
                GetGameInfo2(
                    tableId = gameInitData.tableID
                )
            )
        }
        Assertions.assertInstanceOf(GameInitDataReceivedState::class.java, bot.state)
    }
}