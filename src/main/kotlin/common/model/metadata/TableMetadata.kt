package eelst.ilike.common.model.metadata

data class TableMetadata(
    val tableId: String,
    val players: List<String>,
    val spectators: List<String>,
)
