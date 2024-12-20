package hanablive.bot.state

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.bot.state.GameInitDataReceivedState
import eelst.ilike.hanablive.bot.state.PlayingState
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData
import eelst.ilike.utils.Utils
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import eelst.ilike.common.model.metadata.LocalMirrorMetadataProvider
import eelst.ilike.game.GameData
import eelst.ilike.hanablive.bot.state.CommonState
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class GameDataInitDataReceivedStateTest {
    private val mapper = Utils.jsonObjectMapper

    private val bot = mockk<HanabLiveBot>(relaxed = true)
    private val metadataProvider = LocalMirrorMetadataProvider
    private val gameData = mockk<GameData>()

    private val state = GameInitDataReceivedState(
        bot = bot,
        botPlayerId = "Alice",
        gameInitData = mapper.readValue(Utils.getResourceFileContentAsString("hanablive/gamestarting/game_init_payload.json")),
        variantMetadata = metadataProvider.getVariantMetadata("6 Suits"),
        gameData = gameData,
        commonState = CommonState()
    )

    @Test
    fun `Should switch state to GameStarted`() {
        val initialGameActionListPayloadAsString =
            Utils.getResourceFileContentAsString("hanablive/gamestarting/game_action_list_payload.json")
        val initialGameActionList: GameActionListData = mapper.readValue(initialGameActionListPayloadAsString)

        runBlocking { state.onGameActionListReceived(initialGameActionList) }

        Assertions.assertInstanceOf(PlayingState::class.java, bot.state)
    }

    @Test
    fun `Should initialize the bot player POV`() {
        val initialGameActionListPayloadAsString =
            Utils.getResourceFileContentAsString("hanablive/gamestarting/game_action_list_payload.json")
        val initialGameActionList: GameActionListData = mapper.readValue(initialGameActionListPayloadAsString)

        runBlocking { state.onGameActionListReceived(initialGameActionList) }

        val botState = bot.state as PlayingState

    }
}