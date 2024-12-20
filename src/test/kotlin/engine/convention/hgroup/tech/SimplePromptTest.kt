package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.SimplePrompt
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class SimplePromptTest {
    @Test
    fun `Should find simple prompts`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(12)

        val actual = SimplePrompt.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                color = Color.BLUE,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.THREE,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.RED,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should avoid wrong prompt`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(13)

        val actual = SimplePrompt.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                color = Color.BLUE,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a wrong prompt if it can be patched in time`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(14)

        val actual = SimplePrompt.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.THREE,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                color = Color.RED,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.BLUE,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a wrong prompt if it cannot be patched in time`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(15)

        val actual = SimplePrompt.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.BLUE,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should recognize a prompt when it given to a teammate`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(26)
        val action = ColorClueAction(
            clueGiver = "Donald",
            clueReceiver = "Bob",
            color = Color.BLUE,
        )

        val actual = SimplePrompt.matches(action, setOf(1), playerPOV)

        Assertions.assertTrue(actual)
    }
}
