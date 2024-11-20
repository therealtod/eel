package engine.player

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.level.Level1
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.engine.convention.hgroup.tech.DiscardChop
import eelst.ilike.engine.convention.hgroup.tech.PlayKnownPlayable
import eelst.ilike.engine.action.Discard
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.action.Play
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test


class OldActivePlayerTest {
    @Test
    fun `Should find all possible actions in the scenario`() {
        val activePlayer = TestUtils.getActivePlayerFromScenario(1)

        val actual = activePlayer.getLegalActions(Level1)

        val expected = setOf(
            ConventionalAction(
                GiveClue(
                    clue = RankClue(Rank.FOUR),
                    to = "Cathy",
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                GiveClue(
                    clue = ColorClue(Color.PURPLE),
                    to = "Cathy",
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                GiveClue(
                    clue = RankClue(Rank.ONE),
                    to = "Cathy",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                GiveClue(
                    clue = ColorClue(Color.BLUE),
                    to = "Cathy",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                GiveClue(
                    clue = RankClue(Rank.ONE),
                    to = "Donald",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                GiveClue(
                    clue = ColorClue(Color.RED),
                    to = "Donald",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                action = Discard(
                    slotIndex = 3,
                ),
                tech = DiscardChop,
            ),
            ConventionalAction(
                action = Play(
                    slotIndex = 4
                ),
                tech = PlayKnownPlayable,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}