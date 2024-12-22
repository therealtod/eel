package hanablive.bot.state

import eelst.ilike.engine.convention.hgroup.level.Level1
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.bot.state.CommonState
import eelst.ilike.hanablive.bot.state.PlayingState
import eelst.ilike.hanablive.model.adapter.HanabLivePlayerPOVAdapter
import eelst.ilike.hanablive.model.dto.PlayActionData
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData
import eelst.ilike.hanablive.model.dto.instruction.GamePlayActionData
import eelst.ilike.hanablive.model.dto.instruction.GameTurnActionData
import eelst.ilike.hanablive.model.dto.instruction.HanabLiveGameAction
import io.mockk.clearAllMocks
import io.mockk.clearMocks
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class PlayingStateTest {
    @BeforeEach
    fun beforeEach() {
        clearMocks(pov)
    }

    @Test
    fun `Should invoke the correct POV method to get the updated POV after receiving a draw action`() {
        val drawActionData = GameDrawActionData(
            playerIndex = 1,
            order = 42,
            suitIndex = 2,
            rank = 3,
        )
        val hanabLiveDrawAction = HanabLiveGameAction(
            tableID = 1234,
            action = drawActionData,
        )
        val hanabLiveTurnAction = HanabLiveGameAction(
            tableID = 1234,
            action = GameTurnActionData(
                num = 10,
                currentPlayerIndex = 2,
            )
        )
        runBlocking { state.onGameAction(hanabLiveDrawAction) }
        runBlocking { state.onGameAction(hanabLiveTurnAction) }

        verify(exactly = 1) { pov.getUpdatedWithDrawAction(drawActionData) }
    }

    @Test
    fun `Should invoke the correct POV method to get the updated POV after receiving a play action`() {
        val playActionData = GamePlayActionData(
            playerIndex = 1,
            order = 42,
            suitIndex = 2,
            rank = 2,
        )
        val hanabLivePlayAction = HanabLiveGameAction(
            tableID = 1234,
            action = playActionData,
        )
        val hanabLiveTurnAction = HanabLiveGameAction(
            tableID = 1234,
            action = GameTurnActionData(
                num = 10,
                currentPlayerIndex = 2,
            )
        )
        runBlocking { state.onGameAction(hanabLivePlayAction) }
        runBlocking { state.onGameAction(hanabLiveTurnAction) }

        verify(exactly = 1) { pov.getUpdatedWithPlayAction(playActionData, false, conventionSet) }
    }

    companion object {
        private lateinit var state: PlayingState
        private val bot = mockk<HanabLiveBot>()
        private val conventionSet = Level1
        private val commonState = CommonState(
            tables = mutableMapOf(),
            conventionSet = conventionSet
        )
        private val pov = mockk<HanabLivePlayerPOVAdapter>(relaxed = true)

        @JvmStatic
        @BeforeAll
        fun setUp() {
            state = PlayingState(
                bot = bot,
                commonState = commonState,
                tableId = 1234,
                gamePOV = pov,
            )
        }
    }
}