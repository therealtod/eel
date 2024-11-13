package game

import eelst.ilike.game.TrashPile
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.NoVarGreen
import eelst.ilike.game.entity.suite.NoVarRed
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

internal class TrashPileTest {
    @Test
    fun `Given an empty trash Should return 0 copies for any card`() {
        val trashPile = TrashPile()

        val card1 = HanabiCard(
            suite = NoVarRed,
            rank = Rank.ONE
        )
        val card2 = HanabiCard(
            suite = NoVarGreen,
            rank = Rank.FIVE
        )
        val card3 = HanabiCard(
            suite = NoVarRed,
            rank = Rank.FOUR
        )

        Assertions.assertEquals(trashPile.copiesOf(card1), 0)
        Assertions.assertEquals(trashPile.copiesOf(card2), 0)
        Assertions.assertEquals(trashPile.copiesOf(card3), 0)
    }

    @Test
    fun `Given a populated Should return the correct amount of copies for each card`() {
        val trashPile = TrashPile(
            cards = listOf(
                HanabiCard(
                    suite = NoVarRed,
                    rank = Rank.ONE
                ),
                HanabiCard(
                    suite = NoVarRed,
                    rank = Rank.ONE
                ),
                HanabiCard(
                    suite = NoVarGreen,
                    rank = Rank.FIVE
                ),
                HanabiCard(
                    suite = NoVarRed,
                    rank = Rank.FOUR
                )
            )
        )


        val card1 = HanabiCard(
            suite = NoVarRed,
            rank = Rank.ONE
        )
        val card2 = HanabiCard(
            suite = NoVarGreen,
            rank = Rank.FIVE
        )
        val card3 = HanabiCard(
            suite = NoVarRed,
            rank = Rank.FOUR
        )
        val card4 = HanabiCard(
            suite = NoVarRed,
            rank = Rank.TWO
        )

        Assertions.assertEquals(trashPile.copiesOf(card1), 2)
        Assertions.assertEquals(trashPile.copiesOf(card2), 1)
        Assertions.assertEquals(trashPile.copiesOf(card3), 1)
        Assertions.assertEquals(trashPile.copiesOf(card4), 0)
    }
}