package game.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class DirectPlayClueTest {
    @Test
    fun `Should find the only direct play clue on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(9)

        val actual = DirectPlayClue.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue
            ),
        )
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all direct play clues on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(10)

        val actual = DirectPlayClue.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = ColorClue(
                    color = Color.PURPLE,
                    receiver = "Bob"
                ),
                tech = DirectPlayClue
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.THREE,
                    receiver = "Bob"
                ),
                tech = DirectPlayClue
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.ONE,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.PURPLE,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue
            ),
        )
        Assertions.assertEquals(expected, actual)
    }
}
