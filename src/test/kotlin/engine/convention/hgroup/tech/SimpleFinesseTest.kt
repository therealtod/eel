package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.SimpleFinesse
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class SimpleFinesseTest {
    @Test
    fun `Should find the only available finesse`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(16)

        val actual = SimpleFinesse.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClueAction(Rank.THREE),
                to = "Cathy",
            )
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not give a reverse finesse`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)

        val actual = SimpleFinesse.getGameActions(playerPOV)

        val expected = emptySet<GameAction>()

        Assertions.assertEquals(expected, actual)
    }
}