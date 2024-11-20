package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.DelayedPlayClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class DelayedPlayGiveClueTest {

    @Test
    fun `Should play clue a red 2 Given that red 1 is known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(6)

        val actual = DelayedPlayClue.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(Rank.TWO),
                to = "Bob"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should play clue a red 4 Given that the entire required sequence is played or known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(7)

        val actual = DelayedPlayClue.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(rank = Rank.FOUR),
                to = "Bob"
            ),
            GiveClue(
                clue = ColorClue(Color.RED),
                to = "Bob"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not play clue red 4 When the required sequence is only partially known`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(8)

        val actual = DelayedPlayClue.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = ColorClue(Color.RED),
                to = "Donald"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}