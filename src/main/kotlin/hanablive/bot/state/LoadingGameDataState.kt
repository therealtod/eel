package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.variant.VariantFactory
import eelst.ilike.hanablive.LobbyState
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.entity.HanabLiveGame
import eelst.ilike.hanablive.entity.dto.instruction.GameActionListData
import eelst.ilike.hanablive.entity.dto.instruction.GameInitData
import eelst.ilike.hanablive.entity.dto.instruction.GetGameInfo2
import eelst.ilike.hanablive.entity.dto.instruction.Loaded

class LoadingGameDataState(
    bot: HanabLiveBot,
    lobbyState: LobbyState,
    ) : HanabLiveBotState(
    bot,
    lobbyState,
) {
        private lateinit var gameInitData: GameInitData

    override suspend fun onGameInitDataReceived(gameInitData: GameInitData) {
        this.gameInitData = gameInitData
        bot.sendHanabLiveInstruction(GetGameInfo2(gameInitData.tableID))
    }

    override suspend fun onGameActionListReceived(gameActionListData: GameActionListData) {
        require(this::gameInitData.isInitialized){
            "The gameActionListData instruction has been received before gameInitData"
        }
        val variantName = gameInitData.options.variantName
        val variantMetadata = bot.getVariantMetadata(variantName)
        val suitIds = variantMetadata.suits
        val suitsMetadata = bot.getSuitsMetadata(suitIds)
        val variant = VariantFactory.createVariant(
            metadata = variantMetadata,
            suitsMetadata = suitsMetadata,
        )
        val game = HanabLiveGame(
            variant = variant,
            playersMetadata = gameInitData.playerNames.mapIndexed { index, name ->
                PlayerMetadata(
                    playerId = name,
                    playerIndex = index
                )
            },
            gameActionListData = gameActionListData,
        )
        val newState = InGameState(
            bot = bot,
            lobbyState = lobbyState,
            game = game,
        )
        switchToState(newState)
        bot.sendHanabLiveInstruction(Loaded(tableId = gameActionListData.tableID))
    }
}
