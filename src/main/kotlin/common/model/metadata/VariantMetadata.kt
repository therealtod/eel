package eelst.ilike.common.model.metadata

data class VariantMetadata(
    val id: Int,
    val newID: String,
    val name: String,
    val suits: List<String>,
    val clueColors: List<Any> = emptyList(),
    val specialRank: Int,
    val specialRankAllClueColors: Boolean = false,
    val specialRankAllClueRanks: Boolean = false,
    val specialRankNoClueColors: Boolean = false,
    val specialRankNoClueRanks: Boolean = false,
    val specialRankDeceptive: Boolean = false,
    val criticalRank: Int? = null,
    val clueStarved: Boolean = false,
    val colorCluesTouchNothing: Boolean = false,
    val rankCluesTouchNothing: Boolean = false,
    val alternatingClues: Boolean = false,
    val cowAndPig: Boolean = false,
    val duck: Boolean = false,
    val oddsAndEvens: Boolean = false,
    val synesthesia: Boolean = false,
    val upOrDown: Boolean = false,
    val throwItInAHole: Boolean = false,
    val funnels: Boolean = false,
    val chimneys: Boolean = false,
    val sudoku: Boolean = false,
    val stackSize: Int = 5,
    val clueRanks: List<Int> = emptyList(),
)
