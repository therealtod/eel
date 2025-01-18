package game.entity

import testcommon.CommonData
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.exception.IllegalGameActionException
import game.entity.suit.*
import game.entity.variant.NoVariant
import io.mockk.*
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

internal class GloballyAvailableGameStateDataTest {
    @BeforeEach
    fun beforeEach() {
        clearAllMocks()
    }

    @Test
    fun `Should add a playable card to the playing stacks`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Blue, rank = Rank.ONE)
        val updatedGame = data.getAfterPlaying(playAction, card)

        val expected = listOf(card)
        val actual = updatedGame.playingStacks[Blue.id]!!.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not add a card to the playing stacks When it's not playable`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)

        data.getAfterPlaying(playAction, card)

        val expected = PlayingStack(cards = emptyList(), suit = Blue)
        val actual = data.playingStacks[Blue.id]
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should discard the misplayed card after a strike`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)
        val updatedGame = data.getAfterPlaying(playAction, card)

        val expected = trashPileCards + card
        val actual = updatedGame.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of strike tokens after a strike`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Blue, rank = Rank.THREE)
        val updatedGame = data.getAfterPlaying(playAction, card)

        val expected = 1
        val actual = updatedGame.strikes

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of clue tokens When successfully playing the last card of a suit`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Purple, rank = Rank.FIVE)
        val updatedGame = data.getAfterPlaying(playAction, card)

        val expected = 6
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not increase the number of clue tokens When a suit is completed And the clue count is 8`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Purple, rank = Rank.FIVE)
        val updatedGame = dataWith8Clues.getAfterPlaying(playAction, card)

        val expected = 8
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should add a discarded card to the discard pile`() {
        val discardAction = DiscardAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        val updatedGame = data.getAfterDiscarding(discardAction, card)

        val expected = trashPileCards + card
        val actual = updatedGame.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should now allow to discard cards When the clue count is maxed`() {
        val discardAction = DiscardAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        Assertions.assertThrows(IllegalGameActionException::class.java) {
            dataWith8Clues.getAfterDiscarding(discardAction, card)
        }
    }

    @Test
    fun `Should increase the number of clue tokens When a card is discarded And the clue count is lower than 8`() {
        val discardAction = DiscardAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val card = HanabiCard(suit = Red, rank = Rank.FIVE)
        val updatedGame = data.getAfterDiscarding(discardAction, card)

        val expected = 6
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should decrease the number of clue tokens When a clue is given`() {
        val clueAction = ClueAction(
            clueGiver = CommonData.aliceMetadata,
            clueReceiver = CommonData.bobMetadata,
            value = Color.RED,
        )
        val updatedGame = data.getAfterClueGiven(clueAction, touchedSlotIndexes = listOf(1))

        val expected = 4
        val actual = updatedGame.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not allow to give a clue When there are no clue tokens left`() {
        val clueAction = ClueAction(
            clueGiver = CommonData.aliceMetadata,
            clueReceiver = CommonData.bobMetadata,
            value = Color.RED,
        )
        Assertions.assertThrows(IllegalGameActionException::class.java) {
            dataWith0Clues.getAfterClueGiven(clueAction, touchedSlotIndexes = listOf(1))
        }
    }

    @Test
    fun `Should compute the correct current deck size`() {
        val expected = 25
        val actual = data.getCurrentDeckSize()

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
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val expected = "Bob"
        val actual = data.getAfterPlay(playAction).getPlayerOnTurn().getMetadata().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a card is played`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val expected = "Bob"
        val actual = data.getAfterPlaying(playAction, HanabiCard(Red, Rank.FOUR))
            .getPlayerOnTurn()
            .getMetadata().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn wrapping around`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.cathyMetadata,
            slotIndex = 1,
        )
        var updatedData = data
        repeat(3) {
            updatedData = updatedData.getAfterPlay(playAction)
        }
        val expected = "Alice"
        val actual = updatedData.getPlayerOnTurn().getMetadata().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a non specified card is discarded`() {
        val discardAction = DiscardAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val expected = "Bob"
        val actual = data.getAfterDiscard(discardAction).getPlayerOnTurn().getMetadata().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a card is discarded`() {
        val discardAction = DiscardAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )
        val expected = "Bob"
        val actual = data.getAfterDiscarding(discardAction, HanabiCard(Red, Rank.FOUR))
            .getPlayerOnTurn().getMetadata()
            .playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should rotate the player on turn When a clue is given`() {
        val clueAction = ClueAction(
            clueGiver = CommonData.aliceMetadata,
            clueReceiver = CommonData.bobMetadata,
            value = Color.RED,
        )
        val expected = "Bob"
        val actual = data.getAfterClueGiven(clueAction, listOf(1)).getPlayerOnTurn().getMetadata().playerId

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should update the action executor when a card is drawn`() {
        val drawAction = DrawAction(CommonData.aliceMetadata)

        data.getAfterDraw(drawAction)

        verify(exactly = 1) { players.first().getUpdatedAfterDrawing(any()) }
    }

    @Test
    fun `Should update the action executor when a card is played`() {
        val playAction = PlayAction(
            playerMetadata = CommonData.aliceMetadata,
            slotIndex = 1,
        )

        data.getAfterPlay(playAction)

        verify(exactly = 1) { players.first().getUpdatedAfterPlaying(1) }
    }

    @Test
    fun `Should update the action executor when a card is discarded`() {
        val discardAction = DiscardAction(
            playerMetadata = CommonData.bobMetadata,
            slotIndex = 3,
        )

        data.getAfterDiscard(discardAction)

        verify(exactly = 1) { players[1].getUpdatedAfterDiscarding(3) }
    }

    @Test
    fun `Should update the clue receiver when a clue is given`() {
        val clueAction = ClueAction(
            clueGiver = CommonData.aliceMetadata,
            clueReceiver = CommonData.cathyMetadata,
            value = Color.YELLOW
        )

        data.getAfterClueGiven(clueAction,listOf(1))

        verify(exactly = 1) { players.last().getUpdatedAfterClueGiven(Color.YELLOW, listOf(1)) }
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


        private val playersMetadata = listOf(
            CommonData.aliceMetadata,
            CommonData.bobMetadata,
            CommonData.cathyMetadata,
        )
        private val players = playersMetadata.map {
            spyk(
                Player(
                    playerMetadata = it,
                    hand = List(5) { mockk(relaxed = true) },
                )
            )
        }

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
                currentDeckSize = 25,
                players = players,
            )
            dataWith8Clues = data.copy(clueTokens = 8)
            dataWith0Clues = data.copy(clueTokens = 0)
        }
    }
}
