package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.TwoSave
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class TwoSaveTest {
    @Test
    fun `Should save the only visible copy of y2`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)

        val actual = TwoSave.getGameActions(playerPOV)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.TWO,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save Cathy's y2 When the other copy is visible in Bob's hand`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(18)

        val actual = TwoSave.getGameActions(playerPOV)

        val expected = emptySet<GameAction>()

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should save either Bob's or Cathy's y2 when both copies are on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(19)

        val actual = TwoSave.getGameActions(playerPOV,)

        val expected = setOf(
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Bob",
                rank = Rank.TWO,
            ),
            RankClueAction(
                clueGiver = "Alice",
                clueReceiver = "Cathy",
                rank = Rank.TWO,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should recognise a two save when given to a teammate`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Cathy",
            rank = Rank.TWO,
        )

        val actual = TwoSave.matches(action, setOf(1, 4, 5), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should recognise a two save when receiving it`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)
        val action = RankClueAction(
            clueGiver = "Cathy",
            clueReceiver = "Alice",
            rank = Rank.TWO,
        )

        val actual = TwoSave.matches(action, setOf(4, 5), playerPOV)

        Assertions.assertTrue(actual)
    }

    @Test
    fun `Should notice that a 2 clue on chop is not a 2 save if the other copy is visible by te clue giver`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(27)
        val action = RankClueAction(
            clueGiver = "Donald",
            clueReceiver = "Cathy",
            rank = Rank.TWO,
        )

        val actual = TwoSave.matches(action, setOf(2, 4),
            playerPOV)

        Assertions.assertFalse(actual)
    }

    @Test
    fun `Should not classify a 2 clue as a 2 save if if the focus is not the chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)
        val action = RankClueAction(
            clueGiver = "Cathy",
            clueReceiver = "Alice",
            rank = Rank.TWO,
        )

        val actual = TwoSave.matches(action, setOf(4), playerPOV)

        Assertions.assertFalse(actual)
    }

    @Test
    fun `Should not give a 2 save if the same card is fully known in the player's hand`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(28)

        val actual = TwoSave.getGameActions(playerPOV)

        val expected = emptySet<ClueAction>()

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not think a 2 clue to a teammate is a 2 save if the same card is fully known in the player's hand`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(28)
        val action = RankClueAction(
            clueGiver = "Bob",
            clueReceiver = "Cathy",
            rank = Rank.TWO,
        )

        val actual = TwoSave.matches(action, setOf(1, 3, 4, 5), playerPOV)

        Assertions.assertFalse(actual)
    }
}
