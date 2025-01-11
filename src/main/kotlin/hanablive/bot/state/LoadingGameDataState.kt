package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.GameConstants
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.factory.VariantFactory
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.dto.instruction.GameInitData
import eelst.ilike.hanablive.entity.dto.instruction.GetGameInfo2

class LoadingGameDataState(
    bot: HanabLiveBot,
    lobbyState: LobbyState,

    ) : HanabLiveBotState(
    bot,
    lobbyState,
) {
    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        val variantName = gameInitData.options.variantName
        val variantMetadata = bot.getVariantMetadata(variantName)
        val suitIds = variantMetadata.suits
        val suitsMetadata = bot.getSuitsMetadata(suitIds)
        val globallyAvailableGameData = parseGloballyAvailableInfo(
            gameInitData = gameInitData,
            variantMetadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        val newState = InGameState(
            bot = bot,
            lobbyState = lobbyState,
            globallyAvailableGameData = globallyAvailableGameData,
        )
        switchToState(newState)
        bot.sendHanabLiveInstruction(GetGameInfo2(gameInitData.tableID))
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
}
