package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class CriticalSaveTest {
    @Test
    fun `Should return 2 actions which save the only critical card on chop visible on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(1)

        val actual = CriticalSave.getGameActions(playerPOV)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.PURPLE,
            ),
        )
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should return no actions Given there is nothing in the trash`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(2)

        val actual = CriticalSave.getGameActions(playerPOV,)

        val expected = emptySet<GameAction>()
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save 5s on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(3)

        val actual = CriticalSave.getGameActions(playerPOV,)

        val expected = emptySet<GameAction>()
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all critical saves Given a state with multiple critical cards on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(4)

        val actual = CriticalSave.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                color = Color.PURPLE,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Donald",
                rank = Rank.FOUR,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Donald",
                color = Color.YELLOW,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Emily",
                rank = Rank.TWO,
            ),
            ColorClueAction(
                clueGiver = "Alice",
                clueReceiver = "Emily",
                color = Color.RED,
            )
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save critical playables`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(5)

        val actual = CriticalSave.getGameActions(playerPOV,)

        val expected = emptySet<GameAction>()

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should match a clue which saves a critical card on a teammate hand`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(1)

        val action = RankClueAction(
                clueGiver = "Bob",
                clueReceiver = "Cathy",
                rank = Rank.FOUR,
            )

        val actual = CriticalSave.matches(action, setOf(4), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should not recognize a clue as a CriticalSave if the chop cannot be a critical card`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(1)

        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Alice",
            rank = Rank.FOUR,
        )

        val actual = CriticalSave.matches(action, setOf(1, 3), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should recognize a clue as a CriticalSave if the chop can be a critical card`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(21)

        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Alice",
            rank = Rank.FOUR,
        )

        val actual = CriticalSave.matches(action, setOf(1, 3), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should not recognize a clue as a CriticalSave if the focus is not the chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(21)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Alice",
            rank = Rank.FOUR,
        )

        val actual = CriticalSave.matches(action, setOf(1), playerPOV)

        Assertions.assertFalse(actual)
    }
}
