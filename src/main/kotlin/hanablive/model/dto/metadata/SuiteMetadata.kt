package eelst.ilike.hanablive.model.dto.metadata

data class SuiteMetadata(
    val name: String,
    val id: String,
    val displayName: String = name,
    val abbreviation: String = id,
    val fill: String? = null,
    val fillColors: List<String> = emptyList(),
    val clueColors: List<String> = listOf(name),
    val createVariants: Boolean = false,
    val pip: String,
    val prism: Boolean = false,
    val oneOfEach: Boolean = false,
    val allClueColors: Boolean = false,
    val allClueRanks: Boolean = false,
    val noClueColors: Boolean = false,
    val noClueRanks: Boolean = false,
)
