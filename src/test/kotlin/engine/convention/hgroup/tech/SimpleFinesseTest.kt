package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.SimpleFinesse
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class SimpleFinesseTest {
    @Test
    fun `Should find the only available finesse`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(16)

        val actual = SimpleFinesse.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.THREE,
                    receiver = "Cathy",
                ),
                tech = SimpleFinesse,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not give a reverse finesse`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)

        val actual = SimpleFinesse.getActions(playerPOV)

        val expected = emptySet<ConventionalAction>()

        Assertions.assertEquals(expected, actual)
    }
}