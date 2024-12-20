package game.entity

import eelst.ilike.game.DynamicGameData
import eelst.ilike.game.GameData
import eelst.ilike.game.GameDataImpl
import eelst.ilike.game.PlayerMetadata
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*
import eelst.ilike.game.variant.Variant
import io.mockk.every
import io.mockk.mockk
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.Test

class GameDataImplTest {
    @Test
    fun `Should add a playable card to the playing stacks`() {
        val card = HanabiCard(suite = Blue, rank = Rank.ONE)
        val updatedGameData = gameData.getAfterPlaying(card)

        val expected = listOf(card)
        val actual = updatedGameData.playingStacks[Blue.id]!!.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not add a card to the playing stacks When it's not playable`() {
        val card = HanabiCard(suite = Blue, rank = Rank.THREE)

        Assertions.assertThrows(Exception::class.java) {
            gameData.getAfterPlaying(card)
        }
    }

    @Test
    fun `Should discard the misplayed card after a strike`() {
        val card = HanabiCard(suite = Blue, rank = Rank.THREE)
        val updatedGameData = gameData.getAfterPlaying(card)

        val expected = trashPileCards + card
        val actual = updatedGameData.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of strike tokens after a strike`() {
        val card = HanabiCard(suite = Blue, rank = Rank.THREE)
        val updatedGameData = gameData.getAfterPlaying(card)

        val expected = 1
        val actual = updatedGameData.strikes

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of clue tokens When successfully playing the last card of a suit`() {
        val card = HanabiCard(suite = Purple, rank = Rank.FIVE)
        val updatedGameData = gameData.getAfterPlaying(card)

        val expected = 6
        val actual = updatedGameData.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should add a card to the discard pile`() {
        val card = HanabiCard(suite = Red, rank = Rank.FIVE)
        val updatedGameData = gameData.getAfterDiscard(card)

        val expected = trashPileCards + card
        val actual = updatedGameData.trashPile.cards

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should increase the number of clue tokens When a card is discarded And the clue count is lower than 8`() {
        val card = HanabiCard(suite = Red, rank = Rank.FIVE)
        val updatedGameData = gameData.getAfterDiscard(card)

        val expected = 6
        val actual = updatedGameData.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not increase the number of clue tokens When a card is discarded And the clue count is 8`() {
        val card = HanabiCard(suite = Red, rank = Rank.FIVE)
        val updatedGameData = gameDataWith8Clues.getAfterDiscard(card)

        val expected = 8
        val actual = updatedGameData.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should decrease the number of clue tokens When a clue is given`() {
        val updatedGameData = gameData.getAfterClue()

        val expected = 4
        val actual = updatedGameData.clueTokens

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should not allow to give a clue When there are no clue tokens leeft`() {
        Assertions.assertThrows(Exception::class.java) {
            val updatedGameData = gameDataWith0Clues.getAfterClue()
        }
    }

    companion object {
        private val variant = mockk<Variant>()
        private val redStack = PlayingStack(
            cards = listOf(
                HanabiCard(
                    suite = Red,
                    rank = Rank.ONE
                ),
            ),
            suite = Red
        )
        private val yellowStack = PlayingStack(
            cards = listOf(
                HanabiCard(
                    suite = Yellow,
                    rank = Rank.ONE
                ),
                HanabiCard(
                    suite = Yellow,
                    rank = Rank.TWO
                ),
            ),
            suite = Yellow
        )
        private val greenStack = PlayingStack(
            cards = emptyList(),
            suite = Green
        )
        private val blueStack = PlayingStack(
            cards = emptyList(),
            suite = Blue
        )
        private val purpleStack = PlayingStack(
            cards = listOf(
                HanabiCard(
                    suite = Purple,
                    rank = Rank.ONE
                ),
                HanabiCard(
                    suite = Purple,
                    rank = Rank.TWO
                ),
                HanabiCard(
                    suite = Purple,
                    rank = Rank.THREE
                ),
                HanabiCard(
                    suite = Purple,
                    rank = Rank.FOUR
                ),
            ),
            suite = Purple
        )
        private val trashPileCards = listOf(
            HanabiCard(
                suite = Purple,
                rank = Rank.ONE,
            ),
            HanabiCard(
                suite = Red,
                rank = Rank.ONE,
            ),
            HanabiCard(
                suite = Green,
                rank = Rank.FOUR,
            ),
        )
        private val trashPile = TrashPile(trashPileCards)
        private val dynamicGameData = DynamicGameData(
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
        private val players = mapOf(
            "Alice" to PlayerMetadata(
                playerId = "Alice",
                playerIndex = 0,
            ),
            "Bob" to PlayerMetadata(
                playerId = "Bob",
                playerIndex = 1,
            ),
            "Cathy" to PlayerMetadata(
                playerId = "Cathy",
                playerIndex = 2,
            ),
        )
        private lateinit var gameData: GameData
        private lateinit var gameDataWith8Clues: GameData
        private lateinit var gameDataWith0Clues: GameData

        @JvmStatic
        @BeforeAll
        fun setUp(){
            every { variant.suits } returns setOf(Red, Yellow, Green, Blue, Purple)
            gameData = GameDataImpl(
                players = players,
                variant = variant,
                dynamicGameData = dynamicGameData,
            )
            gameDataWith8Clues = GameDataImpl(
                players = players,
                variant = variant,
                dynamicGameData = dynamicGameData.copy(clueTokens = 8),
            )
            gameDataWith0Clues = GameDataImpl(
                players = players,
                variant = variant,
                dynamicGameData = dynamicGameData.copy(clueTokens = 0),
            )
        }
    }
}
