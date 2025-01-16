package game.entity

import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.exception.IllegalGameActionException
import game.entity.suit.*
import game.entity.variant.NoVariant
import io.mockk.every
import io.mockk.mockk
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Test

class GloballyAvailableGameStateDataTest {
    @Test
    fun `Should add a playable card to the playing stacks`() {
        val card = HanabiCard(suit = Blue, rank = Rank.ONE)
        val updatedGame = data.getAfterPlaying(card)

        val expected = listOf(card)
        val actual = updatedGame.playingStacks[Blue.id]!!.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not add a card to the playing stacks When it's not playable`() {
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)

        data.getAfterPlaying(card)

        val expected = PlayingStack(cards = emptyList(), suit = Blue)
        val actual = data.playingStacks[Blue.id]
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should discard the misplayed card after a strike`() {
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)
        val updatedGame = data.getAfterPlaying(card)

        val expected = trashPileCards + card
        val actual = updatedGame.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of strike tokens after a strike`() {
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)
        val updatedGame = data.getAfterPlaying(card)

        val expected = 1
        val actual = updatedGame.strikes

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of clue tokens When successfully playing the last card of a suit`() {
        val card = HanabiCard(suit = Purple, rank = Rank.FIVE)
        val updatedGame = data.getAfterPlaying(card)

        val expected = 6
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not increase the number of clue tokens When a suit is completed And the clue count is 8`() {
        val card = HanabiCard(suit = Purple, rank = Rank.FIVE)
        val updatedGame = dataWith8Clues.getAfterPlaying(card)

        val expected = 8
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should add a discarded card to the discard pile`() {
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        val updatedGame = data.getAfterDiscarding(card)

        val expected = trashPileCards + card
        val actual = updatedGame.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should now allow to discard cards When the clue count is maxed`() {
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        Assertions.assertThrows(IllegalGameActionException::class.java) {
            dataWith8Clues.getAfterDiscarding(card)
        }
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
    fun `Should decrease the number of clue tokens When a clue is given`() {
        val updatedGame = data.getAfterClueGiven()

        val expected = 4
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not allow to give a clue When there are no clue tokens left`() {
        Assertions.assertThrows(IllegalGameActionException::class.java) {
            dataWith0Clues.getAfterClueGiven()
        }
    }

    @Test
    fun `Should compute the correct current deck size`() {
        val hand1 = mockk<Hand>()
        val hand2 = mockk<Hand>()
        val hand3 = mockk<Hand>()
        every { hand1.size } returns 5
        every { hand2.size } returns 5
        every { hand3.size } returns 5
        val hands = listOf(hand1, hand2, hand3)

        val expected = 18
        val actual = data.getCurrentDeckSize(hands)

        Assertions.assertEquals(expected, actual)

    }

    @Test
    fun `Should compute the correct score`() {
        val expected = 7
        val actual = data.score

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a value of 0 as away-value for a card that is immediately playable`() {
        val card = HanabiCard(
            suit = Red,
            rank = Rank.TWO,
        )

        val expected = 0
        val actual = data.getAwayValue(card)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a value of 2 as away-value for a card that is that is 2 from playable`() {
        val card = HanabiCard(
            suit = Blue,
            rank = Rank.ONE,
        )

        val expected = 0
        val actual = data.getAwayValue(card)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should give a value of 0 for the first playable card of the suit When the stack is empty`() {
        val card = HanabiCard(
            suit = Red,
            rank = Rank.FOUR,
        )

        val expected = 2
        val actual = data.getAwayValue(card)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should return true When the card is immediately playable`() {
        val card = HanabiCard(
            suit = Red,
            rank = Rank.TWO,
        )

        val result = data.isImmediatelyPlayable(card)

        Assertions.assertTrue(result)
    }

    @Test
    fun `Should rotate the player on turn When an unspecified card is played`() {
        val expected = "Bob"
        val actual = data.getAfterPlay().getPlayerOnTurn().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a card is played`() {
        val expected = "Bob"
        val actual = data.getAfterPlaying(HanabiCard(Red, Rank.FOUR)).getPlayerOnTurn().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn wrapping around`() {
        var updatedData = data
        repeat(3) {
            updatedData = updatedData.getAfterPlay()
        }
        val expected = "Alice"
        val actual = updatedData.getPlayerOnTurn().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a non specified card is discarded`() {
        val expected = "Bob"
        val actual = data.getAfterDiscard().getPlayerOnTurn().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a card is discarded`() {
        val expected = "Bob"
        val actual = data.getAfterDiscarding(HanabiCard(Red, Rank.FOUR)).getPlayerOnTurn().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a clue is given`() {
        val expected = "Bob"
        val actual = data.getAfterClueGiven().getPlayerOnTurn().playerId

        Assertions.assertEquals(expected, actual)
    }

    companion object {
        private val variant = NoVariant
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

        private lateinit var data: GloballyAvailableGameData
        private lateinit var dataWith8Clues: GloballyAvailableGameData
        private lateinit var dataWith0Clues: GloballyAvailableGameData

        @JvmStatic
        @BeforeAll
        fun setUp() {
            data = GloballyAvailableGameData(
                variant = variant,
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
                amountOfCardsPlayed = 7,
                amountOfCardsDiscarded = trashPile.cards.size,
                playersMetadata = listOf(
                    PlayerMetadata("Alice", 0),
                    PlayerMetadata("Bob", 1),
                    PlayerMetadata("Cathy", 2),
                )
            )
            dataWith8Clues = data.copy(clueTokens = 8)
            dataWith0Clues = data.copy(clueTokens = 0)
        }
    }
}
