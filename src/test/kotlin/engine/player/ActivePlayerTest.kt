package engine.player

import TestUtils
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.level.Level1
import eelst.ilike.engine.convention.hgroup.tech.CriticalSave
import eelst.ilike.engine.convention.hgroup.tech.DirectPlayClue
import eelst.ilike.engine.convention.hgroup.tech.DiscardChop
import eelst.ilike.engine.convention.hgroup.tech.PlayKnownPlayable
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ColorClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.action.RankClueAction
import eelst.ilike.utils.Utils
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test


class ActivePlayerTest {
    @Test
    fun `Should find all possible actions in the scenario`() {
        val activePlayer = TestUtils.getPlayerPOVFromScenario(1)

        val actual = activePlayer.getLegalActions(Level1)

        val expected = setOf(
            ConventionalAction(
                RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    rank = Rank.FOUR,
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                ColorClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    color = Color.PURPLE,
                ),
                tech = CriticalSave,
            ),
            ConventionalAction(
                RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    rank = Rank.ONE,
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                ColorClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Cathy",
                    color = Color.BLUE,
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                RankClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Donald",
                    rank = Rank.ONE,
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                ColorClueAction(
                    clueGiver = "Alice",
                    clueReceiver = "Donald",
                    color = Color.RED,
                ),
                tech = DirectPlayClue,
            ),
            ConventionalAction(
                action = DiscardAction(
                    playerId = "Alice",
                    slotIndex = 3,
                ),
                tech = DiscardChop,
            ),
            ConventionalAction(
                action = PlayAction(
                    playerId = "Alice",
                    slotIndex = 4
                ),
                tech = PlayKnownPlayable,
            ),
        )

        Assertions.assertEquals(expected, actual)
    }
}