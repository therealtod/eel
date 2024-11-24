package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.TwoSave
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
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

        val actual = TwoSave.getGameActions(playerPOV)

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
}
