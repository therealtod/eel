package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.hgroup.tech.SimplePrompt
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class SimplePromptTest {
    @Test
    fun `Should find simple prompts`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(12)

        val actual = SimplePrompt.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(Rank.FOUR),
                from = "Alice",
                to = "Bob",
            ),
            GiveClue(
                clue = ColorClue(Color.BLUE),
                from = "Alice",
                to = "Bob"
            ),
            GiveClue(
                clue = RankClue(Rank.THREE),
                from = "Alice",
                to = "Cathy",
            ),
            GiveClue(
                clue = ColorClue(Color.RED),
                from = "Alice",
                to = "Cathy"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should avoid wrong prompt`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(13)

        val actual = SimplePrompt.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(Rank.FOUR),
                from = "Alice",
                to = "Bob",
            ),
            GiveClue(
                clue = ColorClue(Color.BLUE),
                from = "Alice",
                to = "Bob"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a wrong prompt if it can be patched in time`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(14)

        val actual = SimplePrompt.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(rank = Rank.THREE),
                from = "Alice",
                to = "Bob",
            ),
            GiveClue(
                clue = ColorClue(color = Color.RED),
                from = "Alice",
                to = "Bob"
            ),
            GiveClue(
                clue = RankClue(rank = Rank.FOUR),
                from = "Alice",
                to = "Cathy",
            ),
            GiveClue(
                clue = ColorClue(color = Color.BLUE),
                from = "Alice",
                to = "Cathy"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a wrong prompt if it cannot be patched in time`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(15)

        val actual = SimplePrompt.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(rank = Rank.FOUR),
                from = "Alice",
                to = "Cathy",
            ),
            GiveClue(
                clue = ColorClue(color = Color.BLUE),
                from = "Alice",
                to = "Cathy"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}
