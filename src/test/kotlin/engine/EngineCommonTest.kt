package engine

import eelst.ilike.engine.EngineCommon
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.FiveSave
import eelst.ilike.engine.convention.hgroup.tech.SimpleFinesse
import eelst.ilike.engine.convention.hgroup.tech.SimplePrompt
import eelst.ilike.engine.convention.hgroup.tech.TwoSave
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class EngineCommonTest {
    @Test
    fun `Should prune conflicting actions`() {
        val actions = listOf(
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Bob"
                ),
                tech = SimpleFinesse
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Cathy"
                ),
                tech = TwoSave
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Cathy"
                ),
                tech = SimpleFinesse
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FIVE,
                    receiver = "Bob"
                ),
                tech = FiveSave
            ),
        )

        val actual = EngineCommon.getPrunedAction(actions)
        val expected = setOf(
            ConventionalAction(
                action = ColorClue(
                    color = Color.RED,
                    receiver = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.TWO,
                    receiver = "Cathy"
                ),
                tech = TwoSave
            ),
            ConventionalAction(
                action = RankClue(
                    rank = Rank.FIVE,
                    receiver = "Bob"
                ),
                tech = FiveSave
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}