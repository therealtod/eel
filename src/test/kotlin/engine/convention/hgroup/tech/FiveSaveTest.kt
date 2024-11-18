package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.FiveSave
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import kotlin.test.Test

internal class FiveSaveTest {
    @Test
    fun `Should find the only 5 save on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(11)

        val actual = FiveSave.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FIVE,
                    receiver = "Cathy"
                ),
                tech = FiveSave
            )
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all the 5 saves available on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(20)

        val actual = FiveSave.getActions(playerPOV)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FIVE,
                    receiver = "Bob"
                ),
                tech = FiveSave
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FIVE,
                    receiver = "Cathy"
                ),
                tech = FiveSave
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FIVE,
                    receiver = "Donald"
                ),
                tech = FiveSave
            )
        )

        Assertions.assertEquals(expected, actual)
    }
}