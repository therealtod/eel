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
import eelst.ilike.game.entity.action.ColorClue
import eelst.ilike.game.entity.action.RankClue
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class EngineCommonTest {
    @Test
    fun `Should prune conflicting actions`() {
        val actions = listOf(
            ConventionalAction(
                action = GiveClue(
                    ColorClue(Color.RED),
                    from = "Alice",
                    to = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = GiveClue(
                    ColorClue(Color.RED),
                    from = "Alice",
                    to = "Bob"
                ),
                tech = SimpleFinesse
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClue(Rank.TWO),
                    from = "Alice",
                    to = "Cathy"
                ),
                tech = TwoSave
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClue(Rank.TWO),
                    from = "Alice",
                    to = "Cathy"
                ),
                tech = SimpleFinesse
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClue(Rank.FIVE),
                    from = "Alice",
                    to = "Bob"
                ),
                tech = FiveSave
            ),
        )

        val actual = EngineCommon.getPrunedAction(actions)
        val expected = setOf(
            ConventionalAction(
                action = GiveClue(
                    ColorClue(Color.RED),
                    from = "Alice",
                    to = "Bob"
                ),
                tech = SimplePrompt
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClue(Rank.TWO),
                    from = "Alice",
                    to = "Cathy"
                ),
                tech = TwoSave
            ),
            ConventionalAction(
                action = GiveClue(
                    RankClue(Rank.FIVE),
                    from = "Alice",
                    to = "Bob"
                ),
                tech = FiveSave
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}