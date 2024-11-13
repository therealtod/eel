package game.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.TwoSave
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class TwoSaveTest {
    @Test
    fun `Should save the only visible copy of y2`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(17)

        val actual = TwoSave.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                RankClue(rank = Rank.TWO, receiver = "Cathy"),
                tech = TwoSave
            )
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save Cathy's y2 When the other copy is visible in Bob's hand`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(18)

        val actual = TwoSave.getActions(playerPOV)

        val expected = emptySet<ConventionalAction>()

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should save either Bob's or Cathy's y2 when both copies are on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(19)

        val actual = TwoSave.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                RankClue(rank = Rank.TWO, receiver = "Bob"),
                tech = TwoSave
            ),
            ConventionalAction(
                RankClue(rank = Rank.TWO, receiver = "Cathy"),
                tech = TwoSave
            )

        )

        Assertions.assertEquals(expected, actual)
    }
}
