package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.DelayedPlayClue
import eelst.ilike.engine.action.ColorClue
import eelst.ilike.engine.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class DelayedPlayClueTest {

    @Test
    fun `Should play clue a red 2 Given that red 1 is known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(6)

        val actual = DelayedPlayClue.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Bob"
                ),
                tech = DelayedPlayClue,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should play clue a red 4 Given that the entire required sequence is played or known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(7)

        val actual = DelayedPlayClue.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Bob"
                ),
                tech = DelayedPlayClue,
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Bob"
                ),
                tech = DelayedPlayClue,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not play clue red 4 When the required sequence is only partially known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(8)

        val actual = DelayedPlayClue.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Donald"
                ),
                tech = DelayedPlayClue,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}