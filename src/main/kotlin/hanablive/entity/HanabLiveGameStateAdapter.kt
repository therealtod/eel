package eelst.ilike.hanablive.entity

import eelst.ilike.game.gamestate.GameState
import eelst.ilike.hanablive.HanabLiveDataParser
import eelst.ilike.hanablive.entity.dto.instruction.*
import engine.card.CardLocationDictionary
import org.apache.logging.log4j.kotlin.Logging

class HanabLiveGameStateAdapter(
    private val gameState: GameState,
    private val cardLocationDictionary: CardLocationDictionary,
): GameState by gameState, Logging {
    fun getAfter(drawActionData: GameDrawActionData, parser: HanabLiveDataParser): HanabLiveGameStateAdapter {
        val drawAction = parser.parseDrawAction(drawActionData)
        val newCardLocationDictionary = cardLocationDictionary.addCard(drawActionData.playerIndex, drawActionData.order)
        val newGameState = if (drawActionData.suitIndex < 0) {
            gameState.getAfter(drawAction)
        } else {
            val parsedCard = parser.parseCardIdentity(
                suitIndex = drawActionData.suitIndex,
                rank = drawActionData.rank
            )
            gameState.getAfter(drawAction, parsedCard)
        }
        return HanabLiveGameStateAdapter(
            newGameState,
            newCardLocationDictionary,
        )
    }

    fun getAfter(playActionData: GamePlayActionData, parser: HanabLiveDataParser): HanabLiveGameStateAdapter {
        val playAction = parser.parsePlayAction(playActionData, cardLocationDictionary)
        val newCardLocationDictionary = cardLocationDictionary
            .removeCard(playActionData.playerIndex, playActionData.order)
        val newGameState = if (playActionData.suitIndex < 0) {
            gameState.getAfter(playAction)
        } else {
            val parsedCard = parser.parseCardIdentity(
                suitIndex = playActionData.suitIndex,
                rank = playActionData.rank
            )
            gameState.getAfter(playAction, parsedCard)
        }
        return HanabLiveGameStateAdapter(
            newGameState,
            newCardLocationDictionary,
        )
    }

    fun getAfter(discardActionData: GameDiscardActionData, parser: HanabLiveDataParser): HanabLiveGameStateAdapter {
        val discardAction = parser.parseDiscardAction(discardActionData, cardLocationDictionary)
        val newCardLocationDictionary = cardLocationDictionary
            .removeCard(discardActionData.playerIndex, discardActionData.order)
        val newGameState = if (discardActionData.suitIndex < 0) {
            gameState.getAfter(discardAction)
        } else {
            val parsedCard = parser.parseCardIdentity(
                suitIndex = discardActionData.suitIndex,
                rank = discardActionData.rank
            )
            gameState.getAfter(discardAction, parsedCard)
        }
        return HanabLiveGameStateAdapter(
            newGameState,
            newCardLocationDictionary,
        )
    }

    fun getAfter(clueActionData: GameClueActionData, parser: HanabLiveDataParser): HanabLiveGameStateAdapter {
        val clueAction = parser.parseClueAction(clueActionData, cardLocationDictionary)
        val newGameState = getAfter(
            clueAction = clueAction,
            touchedSlotsIndexes = clueActionData
                .list
                .map { cardLocationDictionary.getPlayerSlotIndex(playerIndex = clueActionData.target, it) }
        )
        return HanabLiveGameStateAdapter(
            newGameState,
            cardLocationDictionary = cardLocationDictionary,
        )
    }
}
