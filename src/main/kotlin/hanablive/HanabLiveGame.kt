package eelst.ilike.hanablive

import eelst.ilike.game.BaseGame
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant
import eelst.ilike.hanablive.model.dto.instruction.GameClueActionData

class HanabLiveGame(
    variant: Variant,
    playingStacks: Map<SuiteId, PlayingStack>,
    trashPile: TrashPile,
    strikes: Int,
    clueTokens: Int,
    players: Map<PlayerId, GloballyAvailablePlayerInfo>,
): BaseGame(
    variant = variant,
    playingStacks = playingStacks,
    trashPile = trashPile,
    strikes = strikes,
    clueTokens = clueTokens,
    players = players
) {
    private val idToColorMap = availableColors.size.downTo(1)
        .associateWith { availableColors.elementAt(it - 1) }

    private val idToRankMap = availableRanks.size.downTo(1)
        .associateWith { availableRanks.elementAt(it - 1) }

    fun getClueValue(clue: GameClueActionData.Clue): ClueValue {
        return when (clue.type) {
            HanabLiveConstants.COLOR_CLUE_TYPE -> idToColorMap[clue.value]!!
                HanabLiveConstants.RANK_CLUE_TYPE -> idToRankMap[clue.value]!!
            else -> throw UnsupportedOperationException("Hanab Live clue type ${clue.type} is unsupported")
        }
    }

    fun getPlayerSlots(playerId: PlayerId, hanabLiveSlotIds: Collection<Int>): Set<Int> {
        TODO()
    }

    fun getPlayerSlot(playerId: PlayerId, hanabLiveSlotId: Int): Int {
        TODO()
    }
}
