package eelst.ilike.game.entity.suit

interface SuitMetadata {
    val name: String
    val id: String
    val displayName: String
    val abbreviation: String
    val fill: String?
    val fillColors: List<String>
    val clueColors: List<String>
    val prism: Boolean
    val oneOfEach: Boolean
    val allClueColors: Boolean
    val allClueRanks: Boolean
    val noClueColors: Boolean
    val noClueRanks: Boolean
}
