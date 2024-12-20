package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class DirectPlayClueTest {
    @Test
    fun `Should find the only direct play clue on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(9)

        val actual = DirectPlayClue.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.TWO,
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
    fun `Should find all direct play clues on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(10)

        val actual = DirectPlayClue.getGameActions(playerPOV,)

        val expected = setOf(
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                color = Color.PURPLE,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.THREE,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.RED,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.TWO,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.PURPLE,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.ONE,
            ),
        )
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should recognise a direct play clue given to a teammate`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(9)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Cathy",
            rank = Rank.TWO,
        )

        val actual = DirectPlayClue.matches(action, setOf(4), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should recognise a direct play clue when receiving it`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(23)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Alice",
            rank = Rank.TWO,
        )

        val actual = DirectPlayClue.matches(action, setOf(3), playerPOV)

        Assertions.assertTrue(actual)
    }
}
