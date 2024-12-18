package eelst.ilike.common.model.metadata

interface VariantMetadata {
    val id: Int
    val newID: String
    val name: String
    val suits: List<String>
    val clueColors: List<Any>
    val specialRank: Int
    val specialRankAllClueColors: Boolean
    val specialRankAllClueRanks: Boolean
    val specialRankNoClueColors: Boolean
    val specialRankNoClueRanks: Boolean
    val specialRankDeceptive: Boolean
    val criticalRank: Int?
    val clueStarved: Boolean
    val colorCluesTouchNothing: Boolean
    val rankCluesTouchNothing: Boolean
    val alternatingClues: Boolean
    val cowAndPig: Boolean
    val duck: Boolean
    val oddsAndEvens: Boolean
    val synesthesia: Boolean
    val upOrDown: Boolean
    val throwItInAHole: Boolean
    val funnels: Boolean
    val chimneys: Boolean
    val sudoku: Boolean
    val stackSize: Int
    val clueRanks: List<Int>
}
