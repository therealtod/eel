package eelst.ilike.hanablive.entity

import eelst.ilike.game.Game
import eelst.ilike.game.GameConstants
import eelst.ilike.game.GameState
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
import eelst.ilike.hanablive.entity.dto.instruction.GameInitData
import eelst.ilike.hanablive.entity.dto.instruction.HanabLiveGameActionData
import eelst.ilike.hanablive.entity.parsed.ParsedGameActionList
import eelst.ilike.hanablive.factory.GameStateFactory

class HanabLiveGame(
    gameInitData: GameInitData,
    gameActionListData: GameActionListData,
    variantMetadata: VariantMetadata,
    suitsMetadata: Map<SuitId, SuitMetadata>,
): Game {
    private val globallyAvailableGameData: GloballyAvailableGameData
    private val parser: HanabLiveDataParser

    init {
        globallyAvailableGameData = parseGloballyAvailableInfo(
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        parser = HanabLiveDataParser(globallyAvailableGameData)
        val categorizedGameActionData = parser.parseGameActionList(gameActionListData.list)
        setInitialGameState(categorizedGameActionData)
    }

    override fun getGloballyAvailableGameData(): GloballyAvailableGameData {
        return globallyAvailableGameData
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard, isStrike: Boolean): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(playAction, playedCard, isStrike)
        gameStates.add(newGameState)
        return this
    }

    override fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(discardAction, discardedCard)
        gameStates.add(newGameState)
        return this
    }

    override fun getAfter(clueAction: ClueAction, touchedSlotIndexes: Set<Int>): Game {
        val currentGameState = gameStates.last()
        val newGameState = currentGameState.getAfter(clueAction, touchedSlotIndexes)
        gameStates.add(newGameState)
        return this
    }

    private fun setInitialGameState(categorizedGameActions: ParsedGameActionList): Game {
        require(gameStates.isEmpty()) {
            "Cannot set the initial state of a game that has been already initialized"
        }
        val initialTeamKnowledge = parser.parseInitialTeamKnowledge(categorizedGameActions)
        val initialPlayersState = parser.parsePlayers(categorizedGameActions)
        /*
        val initialGameState = GameStateFactory.createKnowledgeAwareGameState(
            globallyAvailableGameData = globallyAvailableGameData,
            players = initialPlayersState,
            teamKnowledge = initialTeamKnowledge
        )

         */
        // gameStates.add(initialGameState)
        return this
    }

    private fun parseGloballyAvailableInfo(
        gameInitData: GameInitData,
        variantMetadata: VariantMetadata,
        suitsMetadata: Map<SuitId, SuitMetadata>
    ): GloballyAvailableGameData {
        val variant = VariantFactory.createVariant(variantMetadata, suitsMetadata)
        val suits = variant.getSuits()
        return GloballyAvailableGameData(
            variant = variant,
            playingStacks = suits.associate { it.id to PlayingStack(emptyList(), it) },
            trashPile = TrashPile(),
            strikes = GameConstants.INITIAL_STRIKE_TOKENS_COUNT,
            clueTokens = GameConstants.MAX_CLUE_TOKENS_COUNT,
            numberOfPlayers = gameInitData.playerNames.size,
            amountOfCardsPlayed = 0,
            possibleMaxScore = variantMetadata.stackSize * suits.size,
            playersMetadata = gameInitData.playerNames.mapIndexed { index, name ->
                PlayerMetadata(
                    playerId = name,
                    playerIndex = index
                )
            }
        )
    }
    private val gameStates: MutableList<GameState> = mutableListOf()
    private val actions: MutableList<HanabLiveGameActionData> = mutableListOf()
}
