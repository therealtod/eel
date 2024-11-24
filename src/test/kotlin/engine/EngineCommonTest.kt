package engine

import eelst.ilike.engine.EngineCommon
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
                action = ColorClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Bob",
                    color = Color.RED,
                ),
                tech = SimplePrompt,
            ),
            ConventionalAction(
                action = ColorClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Bob",
                    color = Color.RED,
                ),
                tech = SimpleFinesse,
            ),
            ConventionalAction(
                action = RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    rank = Rank.TWO,
                ),
                tech = TwoSave,
            ),
            ConventionalAction(
                action = RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    rank = Rank.TWO,
                ),
                tech = SimpleFinesse,
            ),
            ConventionalAction(
                action = RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Bob",
                    rank = Rank.FIVE,
                ),
                tech = FiveSave,
            ),
        )

        val actual = EngineCommon.getPrunedAction(actions)
        val expected = setOf(
            ConventionalAction(
                action = ColorClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Bob",
                    color = Color.RED,
                ),
                tech = SimplePrompt,
            ),
            ConventionalAction(
                action = RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    rank = Rank.TWO,
                ),
                tech = TwoSave,
            ),
            ConventionalAction(
                action = RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Bob",
                    rank = Rank.FIVE,
                ),
                tech = FiveSave,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}