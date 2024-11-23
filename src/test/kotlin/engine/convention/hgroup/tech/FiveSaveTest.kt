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

        val actual = FiveSave.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClueAction(Rank.FIVE),
                to = "Cathy",
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all the 5 saves available on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(20)

        val actual = FiveSave.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClueAction(Rank.FIVE),
                to = "Bob",
            ),
            GiveClue(
                clue = RankClueAction(Rank.FIVE),
                to = "Cathy",
            ),
            GiveClue(
                clue = RankClueAction(Rank.FIVE),
                to = "Donald",
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}