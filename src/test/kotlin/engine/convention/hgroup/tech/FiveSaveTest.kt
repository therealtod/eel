package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.FiveSave
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import kotlin.test.Test

internal class FiveSaveTest {
    @Test
    fun `Should find the only 5 save on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(11)

        val actual = FiveSave.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.FIVE,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all the 5 saves available on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(20)

        val actual = FiveSave.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.FIVE,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.FIVE,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Donald",
                rank = Rank.FIVE,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should recognise a 5 save when given to a teammate`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(11)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Cathy",
            rank = Rank.FIVE,
        )

        val actual = FiveSave.matches(action, setOf(1, 5), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should recognise a 5 save when receiving it`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(11)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Alice",
            rank = Rank.FIVE,
        )

        val actual = FiveSave.matches(action, setOf(5), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should not classify a 5 clue as a save if the focus is not the chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(11)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Alice",
            rank = Rank.FIVE,
        )

        val actual = FiveSave.matches(action, setOf(3), playerPOV)

        Assertions.assertFalse(actual)
    }
}