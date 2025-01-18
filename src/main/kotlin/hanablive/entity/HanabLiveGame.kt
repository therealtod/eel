package eelst.ilike.hanablive.entity

import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.variant.Variant
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.entity.dto.instruction.*
import org.apache.logging.log4j.kotlin.Logging

class HanabLiveGame(
    variant: Variant,
    playersMetadata: List<PlayerMetadata>,
    gameActionListData: GameActionListData,
    initialGameState: GameState,
): Logging {
    private val parser: HanabLiveDataParser = HanabLiveDataParser(variant, playersMetadata)

    fun getCurrentGameState(): HanabLiveGameStateAdapter {
        return gameStates.last()
    }

    fun updateWith(drawActionData: GameDrawActionData): HanabLiveGame {
        val currentGameState = getCurrentGameState()
        val newGameState = currentGameState.getAfter(drawActionData, parser)
        gameStates.add(newGameState)
        return this
    }

    fun updateWith(playActionData: GamePlayActionData): HanabLiveGame {
        val currentGameState = getCurrentGameState()
        val newGameState = currentGameState.getAfter(playActionData, parser)
        gameStates.add(newGameState)
        return this
    }

    fun getAfter(strikeActionData: GameStrikeActionData): HanabLiveGame {
        val currentStrikes = getCurrentGameState().globallyAvailableGameData.strikes
        val previousTurnStrikes = gameStates[gameStates.size - 2].globallyAvailableGameData.strikes
        if (currentStrikes != previousTurnStrikes + 1 )
            logger.error("A strike has not been correctly registered!")
        return this
    }

    fun updateWith(discardActionData: GameDiscardActionData): HanabLiveGame {
        val currentGameState = getCurrentGameState()
        val newGameState = currentGameState.getAfter(discardActionData, parser)
        gameStates.add(newGameState)
        return this
    }

    fun updateWith(clueActionData: GameClueActionData): HanabLiveGame {
        val currentGameState = getCurrentGameState()
        val newGameState = currentGameState.getAfter(clueActionData, parser)
        gameStates.add(newGameState)
        return this
    }

    fun getAfter(statusActionData: GameStatusActionData, touchedSlotIndexes: Set<Int>): HanabLiveGame {
        TODO()
    }

    fun getAfter(turnActionData: GameTurnActionData): HanabLiveGame {
        TODO()
    }



    /*
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

     */


    private val actions: MutableList<HanabLiveGameActionData> = mutableListOf()
    private val gameStates: MutableList<HanabLiveGameStateAdapter> = mutableListOf()
}
