package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class DirectPlayGiveClueTest {
    @Test
    fun `Should find the only direct play clue on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(9)

        val actual = DirectPlayClue.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClue(Rank.TWO),
                from = "Alice",
                to = "Cathy",
            ),
            GiveClue(
                clue = ColorClue(Color.RED),
                from = "Alice",
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
                clue = ColorClue(Color.PURPLE),
                from = "Alice",
                to = "Bob",
            ),
            GiveClue(
                clue = RankClue(Rank.THREE),
                from = "Alice",
                to = "Bob"
            ),
            GiveClue(
                clue = ColorClue(Color.RED),
                from = "Alice",
                to = "Cathy",
            ),
            GiveClue(
                clue = RankClue(Rank.TWO),
                from = "Alice",
                to = "Cathy"
            ),
            GiveClue(
                clue = ColorClue(Color.PURPLE),
                from = "Alice",
                to = "Cathy",
            ),
            GiveClue(
                clue = RankClue(Rank.ONE),
                from = "Alice",
                to = "Cathy"
            ),
        )
        Assertions.assertEquals(expected, actual)
    }
}
