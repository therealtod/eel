package eelst.ilike.hanablive.model.dto.metadata

import eelst.ilike.common.model.metadata.VariantMetadata

data class HanabLiveVariantMetadata(
    override val id: Int,
    override val newID: String,
    override val name: String,
    override val suits: List<String>,
    override val clueColors: List<String> = emptyList(),
    override val specialRank: Int,
    override val specialRankAllClueColors: Boolean = false,
    override val specialRankAllClueRanks: Boolean = false,
    override val specialRankNoClueColors: Boolean = false,
    override val specialRankNoClueRanks: Boolean = false,
    override val specialRankDeceptive: Boolean = false,
    override val criticalRank: Int? = null,
    override val clueStarved: Boolean = false,
    override val colorCluesTouchNothing: Boolean = false,
    override val rankCluesTouchNothing: Boolean = false,
    override val alternatingClues: Boolean = false,
    override val cowAndPig: Boolean = false,
    override val duck: Boolean = false,
    override val oddsAndEvens: Boolean = false,
    override val synesthesia: Boolean = false,
    override val upOrDown: Boolean = false,
    override val throwItInAHole: Boolean = false,
    override val funnels: Boolean = false,
    override val chimneys: Boolean = false,
    override val sudoku: Boolean = false,
    override val stackSize: Int = 5,
    override val clueRanks: List<Int> = listOf(1, 2, 3, 4, 5),
): VariantMetadata
