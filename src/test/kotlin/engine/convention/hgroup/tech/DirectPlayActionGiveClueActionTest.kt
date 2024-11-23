package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class DirectPlayActionGiveClueActionTest {
    @Test
    fun `Should find the only direct play clue on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(9)

        val actual = DirectPlayClue.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClueAction(Rank.TWO),
                to = "Cathy",
            ),
            GiveClue(
                clue = ColorClueAction(Color.RED),
                to = "Cathy",
            ),
        )
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all direct play clues on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(10)

        val actual = DirectPlayClue.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = ColorClueAction(Color.PURPLE),
                to = "Bob",
            ),
            GiveClue(
                clue = RankClueAction(Rank.THREE),
                to = "Bob"
            ),
            GiveClue(
                clue = ColorClueAction(Color.RED),
                to = "Cathy",
            ),
            GiveClue(
                clue = RankClueAction(Rank.TWO),
                to = "Cathy"
            ),
            GiveClue(
                clue = ColorClueAction(Color.PURPLE),
                to = "Cathy",
            ),
            GiveClue(
                clue = RankClueAction(Rank.ONE),
                to = "Cathy"
            ),
        )
        Assertions.assertEquals(expected, actual)
    }
}
