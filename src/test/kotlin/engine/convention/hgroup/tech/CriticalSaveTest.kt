package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.engine.action.ColorClue
import eelst.ilike.engine.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class CriticalSaveTest {
    @Test
    fun `Should return 2 actions which save the only critical card on chop visible on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(1)

        val actual = CriticalSave.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Cathy"
                ),
                tech = CriticalSave
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.PURPLE,
                    receiver = "Cathy"
                ),
                tech = CriticalSave
            )
        )
        Assertions.assertEquals(actual, expected)
    }

    @Test
    fun `Should return no actions Given there is nothing in the trash`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(2)

        val actual = CriticalSave.getActions(playerPOV)

        val expected = emptySet<ConventionalAction>()
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save 5s on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(3)

        val actual = CriticalSave.getActions(playerPOV)

        val expected = emptySet<ConventionalAction>()
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all critical saves Given a state with multiple critical cards on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(4)

        val actual = CriticalSave.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Cathy"
                ),
                tech = CriticalSave
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.PURPLE,
                    receiver = "Cathy"
                ),
                tech = CriticalSave
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Donald"
                ),
                tech = CriticalSave
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.YELLOW,
                    receiver = "Donald"
                ),
                tech = CriticalSave
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Emily"
                ),
                tech = CriticalSave
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Emily"
                ),
                tech = CriticalSave
            )
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save critical playables`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(5)

        val actual = CriticalSave.getActions(playerPOV)

        val expected = emptySet<ConventionalAction>()

        Assertions.assertEquals(expected, actual)
    }
}
