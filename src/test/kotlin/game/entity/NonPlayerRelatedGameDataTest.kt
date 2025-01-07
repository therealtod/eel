package game.entity

import eelst.ilike.game.NonPlayerRelatedGameData
import eelst.ilike.game.entity.*
import game.entity.suit.*
import io.mockk.every
import io.mockk.mockk
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Test

class NonPlayerRelatedGameDataTest {
    @Test
    fun `Should add a playable card to the playing stacks`() {
        val card = HanabiCard(suit = Blue, rank = Rank.ONE)
        val updatedGame = data.getAfterPlaying(card, variant)

        val expected = listOf(card)
        val actual = updatedGame.playingStacks[Blue.id]!!.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not add a card to the playing stacks When it's not playable`() {
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)

        data.getAfterPlaying(card, variant)

        val expected = PlayingStack(cards = emptyList(), suit = Blue)
        val actual = data.playingStacks[Blue.id]
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should discard the misplayed card after a strike`() {
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)
        val updatedGame = data.getAfterPlaying(card, variant)

        val expected = trashPileCards + card
        val actual = updatedGame.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of strike tokens after a strike`() {
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)
        val updatedGame = data.getAfterPlaying(card, variant)

        val expected = 1
        val actual = updatedGame.strikes

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of clue tokens When successfully playing the last card of a suit`() {
        val card = HanabiCard(suit = Purple, rank = Rank.FIVE)
        val updatedGame = data.getAfterPlaying(card, variant)

        val expected = 6
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should add a card to the discard pile`() {
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        val updatedGame = data.getAfterDiscarding(card)

        val expected = trashPileCards + card
        val actual = updatedGame.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of clue tokens When a card is discarded And the clue count is lower than 8`() {
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        val updatedGame = data.getAfterDiscarding(card)

        val expected = 6
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not increase the number of clue tokens When a card is discarded And the clue count is 8`() {
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        val updatedGame = dataWith8Clues.getAfterDiscarding(card)

        val expected = 8
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should decrease the number of clue tokens When a clue is given`() {
        val updatedGame = data.getAfterPlayerCluing()

        val expected = 4
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not allow to give a clue When there are no clue tokens left`() {
        Assertions.assertThrows(IllegalAccessException::class.java) {
            val updatedGame = dataWith0Clues.getAfterPlayerCluing()
        }
    }

    companion object {
        private val variant = mockk<Variant>()
        private val redStack = PlayingStack(
            cards = listOf(
                HanabiCard(
                    suit = Red,
                    rank = Rank.ONE
                ),
            ),
            suit = Red
        )
        private val yellowStack = PlayingStack(
            cards = listOf(
                HanabiCard(
                    suit = Yellow,
                    rank = Rank.ONE
                ),
                HanabiCard(
                    suit = Yellow,
                    rank = Rank.TWO
                ),
            ),
            suit = Yellow
        )
        private val greenStack = PlayingStack(
            cards = emptyList(),
            suit = Green
        )
        private val blueStack = PlayingStack(
            cards = emptyList(),
            suit = Blue
        )
        private val purpleStack = PlayingStack(
            cards = listOf(
                HanabiCard(
                    suit = Purple,
                    rank = Rank.ONE
                ),
                HanabiCard(
                    suit = Purple,
                    rank = Rank.TWO
                ),
                HanabiCard(
                    suit = Purple,
                    rank = Rank.THREE
                ),
                HanabiCard(
                    suit = Purple,
                    rank = Rank.FOUR
                ),
            ),
            suit = Purple
        )
        private val trashPileCards = listOf(
            HanabiCard(
                suit = Purple,
                rank = Rank.ONE,
            ),
            HanabiCard(
                suit = Red,
                rank = Rank.ONE,
            ),
            HanabiCard(
                suit = Green,
                rank = Rank.FOUR,
            ),
        )
        private val trashPile = TrashPile(trashPileCards)
        private val data = NonPlayerRelatedGameData(
            playingStacks = mapOf(
                "red" to redStack,
                "yellow" to yellowStack,
                "green" to greenStack,
                "blue" to blueStack,
                "purple" to purpleStack,

                ),
            trashPile = trashPile,
            strikes = 0,
            clueTokens = 5,
        )

        private lateinit var dataWith8Clues: NonPlayerRelatedGameData
        private lateinit var dataWith0Clues: NonPlayerRelatedGameData

        @JvmStatic
        @BeforeAll
        fun setUp() {
            every { variant.suits } returns setOf(Red, Yellow, Green, Blue, Purple)
            dataWith8Clues =  data.copy(clueTokens = 8)

            dataWith0Clues = data.copy(clueTokens = 0)

        }
    }
}