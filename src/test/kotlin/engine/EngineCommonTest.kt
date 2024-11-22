package engine

import eelst.ilike.engine.EngineCommon
import eelst.ilike.engine.action.GiveClue
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.tech.FiveSave
import eelst.ilike.engine.convention.hgroup.tech.SimpleFinesse
import eelst.ilike.engine.convention.hgroup.tech.SimplePrompt
import eelst.ilike.engine.convention.hgroup.tech.TwoSave
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.RankClueAction
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class EngineCommonTest {
    @Test
    fun `Should prune conflicting actions`() {
        val actions = listOf(
            ConventionalAction(
                action = GiveClue(
                    ColorClueAction(Color.RED),
                    to = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = GiveClue(
                    ColorClueAction(Color.RED),
                    to = "Bob"
                ),
                tech = SimpleFinesse
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClueAction(Rank.TWO),
                    to = "Cathy"
                ),
                tech = TwoSave
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClueAction(Rank.TWO),
                    to = "Cathy"
                ),
                tech = SimpleFinesse
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClueAction(Rank.FIVE),
                    to = "Bob"
                ),
                tech = FiveSave
            ),
        )

        val actual = EngineCommon.getPrunedAction(actions)
        val expected = setOf(
            ConventionalAction(
                action = GiveClue(
                    ColorClueAction(Color.RED),
                    to = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClueAction(Rank.TWO),
                    to = "Cathy"
                ),
                tech = TwoSave
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClueAction(Rank.FIVE),
                    to = "Bob"
                ),
                tech = FiveSave
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}