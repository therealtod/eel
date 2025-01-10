package eelst.ilike.hanablive.entity.dto.metadata

import eelst.ilike.game.entity.suit.SuitMetadata

data class HanabLiveSuitMetadata(
    override val name: String,
    override val id: String,
    override val displayName: String = name,
    override val abbreviation: String = id,
    override val fill: String? = null,
    override val fillColors: List<String> = emptyList(),
    override val clueColors: List<String> = listOf(name),
    override val prism: Boolean = false,
    override val oneOfEach: Boolean = false,
    override val allClueColors: Boolean = false,
    override val allClueRanks: Boolean = false,
    override val noClueColors: Boolean = false,
    override val noClueRanks: Boolean = false,
): SuitMetadata
