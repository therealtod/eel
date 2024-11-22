package engine.convention.hgroup.tech

import TestUtils
import eelst.ilike.engine.action.GameAction
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class CriticalSaveTest {
    @Test
    fun `Should return 2 actions which save the only critical card on chop visible on the board`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(1)

        val actual = CriticalSave.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClueAction(Rank.FOUR),
                to = "Cathy"
            ),
            GiveClue(
                clue = ColorClueAction(Color.PURPLE),
                to = "Cathy"
            ),
        )
        Assertions.assertEquals(actual, expected)
    }

    @Test
    fun `Should return no actions Given there is nothing in the trash`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(2)

        val actual = CriticalSave.getGameActions(playerPOV)

        val expected = emptySet<GameAction>()
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save 5s on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(3)

        val actual = CriticalSave.getGameActions(playerPOV)

        val expected = emptySet<GameAction>()
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should find all critical saves Given a state with multiple critical cards on chop`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(4)

        val actual = CriticalSave.getGameActions(playerPOV)

        val expected = setOf(
            GiveClue(
                clue = RankClueAction(Rank.FOUR),
                to = "Cathy"
            ),
            GiveClue(
                clue = ColorClueAction(Color.PURPLE),
                to = "Cathy"
            ),
            GiveClue(
                clue = RankClueAction(Rank.FOUR),
                to = "Donald"
            ),
            GiveClue(
                clue = ColorClueAction(Color.YELLOW),
                to = "Donald"
            ),
            GiveClue(
                clue = RankClueAction(Rank.TWO),
                to = "Emily"
            ),
            GiveClue(
                clue = ColorClueAction(Color.RED),
                to = "Emily"
            ),
        )

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not save critical playables`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(5)

        val actual = CriticalSave.getGameActions(playerPOV)

        val expected = emptySet<GameAction>()

        Assertions.assertEquals(expected, actual)
    }
}
