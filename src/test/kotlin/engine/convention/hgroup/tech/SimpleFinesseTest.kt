package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.SimpleFinesse
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class SimpleFinesseTest {
    @Test
    fun `Should find the only available finesse`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(16)

        val actual = SimpleFinesse.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.THREE,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not give a reverse finesse`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)

        val actual = SimpleFinesse.getGameActions(playerPOV,)

        val expected = emptySet<GameAction>()

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should recognise a simple finesse When it is given to a teammate`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(24)
        val action = RankClueAction(
            clueGiver = "Donald",
            clueReceiver = "Cathy",
            rank = Rank.THREE,
        )

        val actual = SimpleFinesse.matches(action, setOf(2), playerPOV)

        Assertions.assertTrue(actual)
    }
}
