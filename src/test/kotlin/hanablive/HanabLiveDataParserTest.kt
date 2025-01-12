package hanablive

import TestUtils
import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.common.Utils
import eelst.ilike.game.GameConstants
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.entity.dto.instruction.*
import eelst.ilike.hanablive.entity.parsed.ParsedGameActionList
import game.entity.suit.*
import game.entity.variant.NoVariant
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test

class HanabLiveDataParserTest {
    @Test
    fun `Should correctly parse a card Given suitIndex and rank`() {
        val expected = HanabiCard(
            suit = Purple,
            rank = Rank.TWO,
        )
        val actual = parser.parseCard(suitIndex = 4, rank = 2)

        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should correctly categorize a hanab live provided list of game actions`() {
        val payload = TestUtils.loadHanabLivePayload("game_action_list_incomplete_game.json")
        val gameActionListData: GameActionListData = Utils.jsonObjectMapper.readValue(payload)
        val actual = parser.parseGameActionList(gameActionListData.list)

        val expectedInitialDrawActions = listOf(
            GameDrawActionData(
                playerIndex = 0,
                order = 0,
                suitIndex = 1,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 0,
                order = 1,
                suitIndex = 2,
                rank = 4,
            ),
            GameDrawActionData(
                playerIndex = 0,
                order = 2,
                suitIndex = 2,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 0,
                order = 3,
                suitIndex = 3,
                rank = 4,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 4,
                suitIndex = 4,
                rank = 5,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 5,
                suitIndex = 2,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 6,
                suitIndex = 1,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 7,
                suitIndex = 3,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 8,
                suitIndex = 0,
                rank = 3,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 9,
                suitIndex = 3,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 10,
                suitIndex = 2,
                rank = 5,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 11,
                suitIndex = 1,
                rank = 4,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 12,
                suitIndex = 4,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 13,
                suitIndex = 2,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 14,
                suitIndex = 1,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 15,
                suitIndex = 3,
                rank = 3,
            ),
        )
        val expectedActionsByTurn = listOf(
            listOf(
                GameClueActionData(
                    clue = GameClueActionData.Clue(type = 0, value = 0),
                    giver = 0,
                    list = listOf(8, 10),
                    target = 2,
                    turn = 0,
                )
            ),
            listOf(
                GameStatusActionData(
                    clues = 7,
                    score = 0,
                    maxScore = 25,
                ),
                GameTurnActionData(
                    num = 1,
                    currentPlayerIndex = 1,
                ),
                GamePlayActionData(
                    playerIndex = 1,
                    order = 7,
                    suitIndex = 3,
                    rank = 1
                ),
            ),
            listOf(
                GameStatusActionData(
                    clues = 7,
                    score = 1,
                    maxScore = 25
                ),
                GameTurnActionData(
                    num = 2,
                    currentPlayerIndex = 2,
                )
            )
        )
        val expected = ParsedGameActionList(
            initialDrawActions = expectedInitialDrawActions,
            actionsByTurn = expectedActionsByTurn,
        )
        Assertions.assertEquals(expected, actual)
    }

    @Test
    fun `Should return the parsed initial team knowledge Given the initial game actions`() {
        val initialDrawActions = listOf(
            GameDrawActionData(
                playerIndex = 0,
                order = 0,
                suitIndex = 1,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 0,
                order = 1,
                suitIndex = 2,
                rank = 4,
            ),
            GameDrawActionData(
                playerIndex = 0,
                order = 2,
                suitIndex = 2,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 0,
                order = 3,
                suitIndex = 3,
                rank = 4,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 4,
                suitIndex = 4,
                rank = 5,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 5,
                suitIndex = 2,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 6,
                suitIndex = 1,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 1,
                order = 7,
                suitIndex = 3,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 8,
                suitIndex = 0,
                rank = 3,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 9,
                suitIndex = 3,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 10,
                suitIndex = 2,
                rank = 5,
            ),
            GameDrawActionData(
                playerIndex = 2,
                order = 11,
                suitIndex = 1,
                rank = 4,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 12,
                suitIndex = 4,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 13,
                suitIndex = 2,
                rank = 2,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 14,
                suitIndex = 1,
                rank = 1,
            ),
            GameDrawActionData(
                playerIndex = 3,
                order = 15,
                suitIndex = 3,
                rank = 3,
            ),
        )
        val actionsByTurn = listOf(
            listOf(
                GameClueActionData(
                    clue = GameClueActionData.Clue(type = 0, value = 0),
                    giver = 0,
                    list = listOf(8, 10),
                    target = 2,
                    turn = 0,
                )
            ),
            listOf(
                GameStatusActionData(
                    clues = 7,
                    score = 0,
                    maxScore = 25,
                ),
                GameTurnActionData(
                    num = 1,
                    currentPlayerIndex = 1,
                ),
                GamePlayActionData(
                    playerIndex = 1,
                    order = 7,
                    suitIndex = 3,
                    rank = 1
                ),
            ),
            listOf(
                GameStatusActionData(
                    clues = 7,
                    score = 1,
                    maxScore = 25
                ),
                GameTurnActionData(
                    num = 2,
                    currentPlayerIndex = 2,
                )
            )
        )
        val parsedGameActionList = ParsedGameActionList(
            initialDrawActions = initialDrawActions,
            actionsByTurn = actionsByTurn,
        )
        val teamKnowledge = parser.parseInitialTeamKnowledge(parsedGameActionList)
        val expectedAliceSlots = mapOf(
            0 to HanabiCard(
                suit = Yellow,
                rank = Rank.TWO,
            ),
            1 to HanabiCard(
                suit = Green,
                rank = Rank.FOUR,
            ),
            2 to HanabiCard(
                suit = Green,
                rank = Rank.TWO,
            ),
            3 to HanabiCard(
                suit = Blue,
                rank = Rank.FOUR,
            ),
        )
        val expectedBobSlots = mapOf(
            0 to HanabiCard(
                suit = Purple,
                rank = Rank.FIVE,
            ),
            1 to HanabiCard(
                suit = Green,
                rank = Rank.ONE,
            ),
            2 to HanabiCard(
                suit = Yellow,
                rank = Rank.ONE,
            ),
            3 to HanabiCard(
                suit = Blue,
                rank = Rank.ONE,
            ),
        )
        val expectedCathySlots = mapOf(
            0 to HanabiCard(
                suit = Red,
                rank = Rank.THREE,
            ),
            1 to HanabiCard(
                suit = Blue,
                rank = Rank.TWO,
            ),
            2 to HanabiCard(
                suit = Green,
                rank = Rank.FIVE,
            ),
            3 to HanabiCard(
                suit = Yellow,
                rank = Rank.FOUR,
            ),
        )
        val expectedDonaldSlots = mapOf(
            0 to HanabiCard(
                suit = Purple,
                rank = Rank.ONE,
            ),
            1 to HanabiCard(
                suit = Green,
                rank = Rank.TWO,
            ),
            2 to HanabiCard(
                suit = Yellow,
                rank = Rank.ONE,
            ),
            3 to HanabiCard(
                suit = Blue,
                rank = Rank.THREE,
            ),
        )
        val aliceKnowledge = teamKnowledge.getPlayerKnowledge("Alice")


        val expected = mapOf(
            "Alice" to expectedAliceSlots,
            "Bob" to expectedBobSlots,
            "Cathy" to expectedCathySlots,
            "Donald" to expectedDonaldSlots
        )
        val actual = aliceKnowledge.getVisiblePlayersCards()

        Assertions.assertEquals(expected, actual)
    }

    companion object {
        private val variant = NoVariant
        private val redStack = PlayingStack(
            suit = Red
        )
        private val yellowStack = PlayingStack(
            suit = Yellow
        )
        private val greenStack = PlayingStack(
            suit = Green
        )
        private val blueStack = PlayingStack(
            suit = Blue
        )
        private val purpleStack = PlayingStack(
            suit = Purple
        )
        private val trashPile = TrashPile()
        private val playersMetadata = listOf(
            PlayerMetadata(
                playerId = "Alice",
                playerIndex = 0,
            ),
            PlayerMetadata(
                playerId = "Bob",
                playerIndex = 1,
            ),
            PlayerMetadata(
                playerId = "Cathy",
                playerIndex = 2,
            ),
            PlayerMetadata(
                playerId = "Donald",
                playerIndex = 3,
            ),
        )

        private val globallyAvailableGameData = GloballyAvailableGameData(
            variant = variant,
            playingStacks = mapOf(
                "red" to redStack,
                "yellow" to yellowStack,
                "green" to greenStack,
                "blue" to blueStack,
                "purple" to purpleStack,

                ),
            trashPile = trashPile,
            strikes = GameConstants.INITIAL_STRIKE_TOKENS_COUNT,
            clueTokens = GameConstants.MAX_CLUE_TOKENS_COUNT,
            numberOfPlayers = 3,
            amountOfCardsPlayed = 7,
            possibleMaxScore = 25,
            playersMetadata = playersMetadata,
        )

        private val parser = HanabLiveDataParser(globallyAvailableGameData)
    }
}
