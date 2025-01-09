package hanablive.bot.state

import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.bot.state.SittingInLobbyState
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class SittingInLobbyStateTest {
    @Disabled
    @Test
    fun `Should send a joinTable message to the server When requested to join a player`() = runTest {
        state.joinPlayer("Alice")

        coVerify(exactly = 1){
            bot.sendHanabLiveInstruction(any())
        }
    }

    companion object {
        private lateinit var bot: HanabLiveBot
        private lateinit var state: SittingInLobbyState

        @JvmStatic
        @BeforeAll
        fun setUp() {
            bot = mockk()
            state = SittingInLobbyState(bot)
        }
    }
}