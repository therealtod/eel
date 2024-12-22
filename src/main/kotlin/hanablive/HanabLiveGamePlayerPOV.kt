package eelst.ilike.hanablive

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.game.*
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.hanablive.model.dto.command.GameActionType
import eelst.ilike.hanablive.model.dto.instruction.*

class HanabLiveGamePlayerPOV(
    private val playerPOV: GameFromPlayerPOV,
    private val playerHandsSlotOrders: Map<Int, Map<Int, Int>>,
): GameFromPlayerPOV by playerPOV {
    private val playerIndexToIdMap = getGameData().players.values.mapIndexed { index, playerMetadata ->
        Pair(index, playerMetadata.playerId)
    }.toMap()
    private val indexToSuitMap = getGameData().suits.mapIndexed { index, suite ->
        Pair(index, suite)
    }.toMap()
    private val indexToColorMap = getGameData().variant.getCluableColors().mapIndexed { index, color ->
        Pair(index, color)
    }.toMap()
    private val indexToRankMap = getGameData().variant.getCluableRanks().mapIndexed { index, rank ->
        Pair(index, rank)
    }.toMap()

    fun getClueValue(clue: GameClueActionData.Clue): ClueValue {
        return when (clue.type) {
            HanabLiveConstants.COLOR_CLUE_TYPE -> indexToColorMap[clue.value]!!
                HanabLiveConstants.RANK_CLUE_TYPE -> indexToRankMap[clue.value]!!
            else -> throw UnsupportedOperationException("Hanab Live clue type ${clue.type} is unsupported")
        }
    }

    override fun getPlayers(): Map<PlayerId, Player> {
        return playerPOV.getPlayers()
    }

    override fun getPlayer(playerId: PlayerId): Player {
        return getPlayers()[playerId] ?: throw NoSuchElementException(
            "No player with id $playerId in this game"
        )
    }

    fun getPlayerSlots(playerId: PlayerId, hanabLiveSlotIds: Collection<Int>): Set<Int> {
        TODO()
    }

    fun getPlayerSlot(playerId: PlayerId, hanabLiveSlotId: Int): Int {
        TODO()
    }

    fun getUpdatedWithDrawAction(drawActionData: GameDrawActionData): HanabLiveGamePlayerPOV {
        val playerId = playerIndexToIdMap[drawActionData.playerIndex]!!
        val newSlot = HanabLiveDataParser.parseSlot(
            activePlayerId = playerPOV.getOwnPlayerId(),
            slotOwnerId = playerId,
            slotIndex = 1,
            draw = drawActionData,
            indexToRankMap = indexToRankMap,
            indexToSuitMap = indexToSuitMap,
            visibleCards = getVisibleCards(),
            suits = getGameData().suits,
        )
        val drawAction = DrawAction(
            playerId = playerId,
            newSlot = HanabLiveDataParser.parseSlot(
                activePlayerId = getOwnPlayerId(),
                slotOwnerId = playerId,
                slotIndex = 1,
                draw = drawActionData,
                indexToSuitMap = indexToSuitMap,
                indexToRankMap = indexToRankMap,
                visibleCards = getVisibleCards(),
                suits = getGameData().suits,
            )
        )
        val updatedPlayerPOV = playerPOV.getAfter(drawAction, newSlot)
        return HanabLiveGamePlayerPOV(
            playerPOV = updatedPlayerPOV,
            playerHandsSlotOrders = TODO()
        )
    }

    fun getUpdatedWithPlayAction(
        gamePlayActionData: GamePlayActionData,
        isStrike: Boolean,
        conventionSet: ConventionSet,
    ): HanabLiveGamePlayerPOV {
        val playedCard = HanabLiveDataParser.parseCard(
            suitIndex = gamePlayActionData.suitIndex,
            rankIndex = gamePlayActionData.rank,
            suitMap = indexToSuitMap,
            rankMap = indexToRankMap,
        )
        val playAction = PlayAction(
            playerId = playerIndexToIdMap[gamePlayActionData.playerIndex]!!,
            slotIndex = getPlayerSlotIndexByOrder(gamePlayActionData.playerIndex, gamePlayActionData.order)
        )
        val newPOV = playerPOV.getAfter(
            playAction = playAction,
            playedCard = playedCard,
            isStrike = isStrike,
            conventionSet = conventionSet,
        )
        return HanabLiveGamePlayerPOV(
            playerPOV = newPOV,
            playerHandsSlotOrders = TODO()
        )
    }

    fun getUpdatedWithDiscardAction(
        discardActionData: GameDiscardActionData,
        conventionSet: ConventionSet,
    ): HanabLiveGamePlayerPOV {
        val discardedCard = HanabLiveDataParser.parseCard(
            suitIndex = discardActionData.suitIndex,
            rankIndex = discardActionData.rank,
            suitMap = indexToSuitMap,
            rankMap = indexToRankMap,
        )
        val discardAction = DiscardAction(
            playerId = playerIndexToIdMap[discardActionData.playerIndex]!!,
            slotIndex = getPlayerSlotIndexByOrder(discardActionData.playerIndex, discardActionData.order)
        )
        val newPOV = playerPOV.getAfter(
            discardAction = discardAction,
            discardedCard = discardedCard,
            conventionSet = conventionSet,
        )
        return HanabLiveGamePlayerPOV(
            playerPOV = newPOV,
            playerHandsSlotOrders = TODO()
        )
    }

    fun getUpdatedWithClueAction(
        clueActionData: GameClueActionData,
        conventionSet: ConventionSet,
    ): HanabLiveGamePlayerPOV {
        TODO()
    }

    private fun getPlayerSlotIndexByOrder(playerIndex: Int, order: Int): Int {
        val playerSlotOrders = playerHandsSlotOrders[playerIndex]
            ?: throw NoSuchElementException("No slot order information about player with index $playerIndex")
        return playerSlotOrders[order]
            ?: throw NoSuchElementException(
                "No slot corresponding slot for order value $order on player with index $playerIndex"
            )
    }
}
