package eelst.ilike.hanablive.model.dto.metadata

import eelst.ilike.common.model.metadata.SuitMetadata

data class HanabLiveSuiteMetadata(
    override val name: String,
    override val id: String,
    override val displayName: String = name,
    override val abbreviation: String = id,
    override val fill: String? = null,
    override val fillColors: List<String> = emptyList(),
    override val clueColors: List<String> = listOf(name),
    override val createVariants: Boolean = false,
    override val pip: String,
    override val prism: Boolean = false,
    override val oneOfEach: Boolean = false,
    override val allClueColors: Boolean = false,
    override val allClueRanks: Boolean = false,
    override val noClueColors: Boolean = false,
    override val noClueRanks: Boolean = false,
): SuitMetadata
