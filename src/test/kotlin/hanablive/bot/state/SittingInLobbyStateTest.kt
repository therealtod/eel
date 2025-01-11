package hanablive.bot.state

import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.bot.state.SittingInLobbyState
import eelst.ilike.hanablive.entity.TableId
import eelst.ilike.hanablive.entity.dto.instruction.ChatPM
import hanablive.entity.dto.Table
import hanablive.entity.dto.instruction.TableJoin
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import java.util.concurrent.ConcurrentHashMap

class SittingInLobbyStateTest {
    @Test
    fun `Should send a joinTable message to the server When requested to join a player`() = runTest {
        val requester = "Alice"
        state.joinPlayer(requester)

        coVerify(exactly = 1) { bot.sendHanabLiveInstruction(TableJoin(TABLE_ID)) }
    }

    @Disabled
    @Test
    fun `Should transition to TableJoinedAsPlayerState When successfully joining the table`() = runTest {
        val requester = "Alice"
        state.joinPlayer(requester)

        val expectedClass = TableJoinedAsPlayerState::class.java
    }

    @Test
    fun `Should send a PM to the requester When the table to join could not be determined`() = runTest {
        val requester = "Donald"
        state.joinPlayer(requester)

        coVerify(exactly = 1) { bot.sendHanabLiveInstruction(any<ChatPM>()) }
    }

    companion object {
        private val bot = mockk<HanabLiveBot>(relaxed = true)
        private lateinit var state: SittingInLobbyState

        private const val TABLE_ID = 1234
        private val table = mockk<Table>()

        @JvmStatic
        @BeforeAll
        fun setUp() {
            every { table.players } returns listOf("Alice", "Bob", "Cathy")
            val tables = ConcurrentHashMap<TableId, Table>()
            tables[TABLE_ID] = table
            val lobbyState = LobbyState(tables)
            state = SittingInLobbyState(bot, lobbyState)
        }
    }
}
