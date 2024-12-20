package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.DelayedPlayClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.fail

internal class DelayedPlayClueTest {
    @Test
    fun `Should play clue a red 2 Given that red 1 is known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(6)

        val actual = DelayedPlayClue.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.TWO,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should play clue a red 4 Given that the entire required sequence is played or known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(7)

        val actual = DelayedPlayClue.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                color = Color.RED,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not play clue red 4 When the required sequence is only partially known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(8)

        val actual = DelayedPlayClue.getGameActions(playerPOV,)

        val expected = setOf(
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Donald",
                color = Color.RED,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should recognize a delayed play clue when given to a teammate`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(7)
        val action = ColorClueAction(
            clueGiver = "Cathy",
            clueReceiver = "Bob",
            color = Color.RED,
        )


        val actual = DelayedPlayClue.matches(action, setOf(1, 3), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should recognize a delayed play clue when receiving it`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(22)
        val action = ColorClueAction(
            clueGiver = "Cathy",
            clueReceiver = "Alice",
            color = Color.RED,
        )

        val actual = DelayedPlayClue.matches(action, setOf(1, 3), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should acknowledge that the focused card can be playable via the already globally known cards`() {
        fail(NotImplementedError())
    }
}