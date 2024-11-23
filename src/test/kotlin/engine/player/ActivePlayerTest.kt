package engine.player

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.level.Level1
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.engine.convention.hgroup.tech.DiscardChop
import eelst.ilike.engine.convention.hgroup.tech.PlayKnownPlayable
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test


class ActivePlayerTest {
    @Test
    fun `Should find all possible actions in the scenario`() {
        val activePlayer = TestUtils.getActivePlayerFromScenario(1)

        val actual = activePlayer.getLegalActions(Level1)

        val expected = setOf(
            ConventionalAction(
                GiveClue(
                    clue = RankClueAction(Rank.FOUR),
                    to = "Cathy",
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                GiveClue(
                    clue = ColorClueAction(Color.PURPLE),
                    to = "Cathy",
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                GiveClue(
                    clue = RankClueAction(Rank.ONE),
                    to = "Cathy",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                GiveClue(
                    clue = ColorClueAction(Color.BLUE),
                    to = "Cathy",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                GiveClue(
                    clue = RankClueAction(Rank.ONE),
                    to = "Donald",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                GiveClue(
                    clue = ColorClueAction(Color.RED),
                    to = "Donald",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                action = DiscardAction(
                    slotIndex = 3,
                ),
                tech = DiscardChop,
            ),
            ConventionalAction(
                action = PlayAction(
                    slotIndex = 4
                ),
                tech = PlayKnownPlayable,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}