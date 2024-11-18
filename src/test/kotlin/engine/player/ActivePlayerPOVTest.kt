package engine.player

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.level.Level1
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.engine.convention.hgroup.tech.DiscardChop
import eelst.ilike.engine.convention.hgroup.tech.PlayKnownPlayable
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.Discard
import eelst.ilike.game.action.Play
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test


class ActivePlayerPOVTest {
    @Test
    fun `Should find all possible actions in the scenario`() {
        val playerPOV = TestUtils.getPlayerPOVFromScenario(1)

        val actual = playerPOV.getActions(Level1)

        val expected = setOf(
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FOUR,
                    receiver = "Cathy"
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.PURPLE,
                    receiver = "Cathy",
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.ONE,
                    receiver = "Cathy"
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.BLUE,
                    receiver = "Cathy",
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.ONE,
                    receiver = "Donald"
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Donald",
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