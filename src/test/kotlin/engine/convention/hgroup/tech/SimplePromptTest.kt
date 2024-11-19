package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.SimplePrompt
import eelst.ilike.engine.action.ColorClue
import eelst.ilike.engine.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class SimplePromptTest {
    @Test
    fun `Should find simple prompts`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(12)

        val actual = SimplePrompt.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.BLUE,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.THREE,
                    receiver = "Cathy"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Cathy"
                ),
                tech = SimplePrompt
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should avoid wrong prompt`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(13)

        val actual = SimplePrompt.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.BLUE,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a wrong prompt if it can be patched in time`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(14)

        val actual = SimplePrompt.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.THREE,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Cathy"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.BLUE,
                    receiver = "Cathy"
                ),
                tech = SimplePrompt
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a wrong prompt if it cannot be patched in time`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(15)

        val actual = SimplePrompt.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Cathy"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.BLUE,
                    receiver = "Cathy"
                ),
                tech = SimplePrompt
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}
