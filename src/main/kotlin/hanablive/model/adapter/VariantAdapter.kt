package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.variant.Variant
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata

class VariantAdapter(
    private val variantMetadata: VariantMetadata,
    suitsMetadata: Collection<SuiteMetadata>,
) : Variant(
    id = variantMetadata.id.toString(),
    name = variantMetadata.name,
    suits = variantMetadata.suits.map { variantSuiteId ->
        SuiteAdapter(
            suiteMetadata = suitsMetadata.find { it.id == variantSuiteId }
                ?: throw IllegalStateException("Could not find the metadata for the suite $variantSuiteId"),
            variantMetadata = variantMetadata,
        )
    }.toSet()
) {

    override fun getCluableRanks(): Set<Rank> {
        return variantMetadata.clueRanks.map { Rank.getByNumericalValue(it) }.toSet()
    }

    override fun getCluableColors(): Set<Color> {
        TODO()
    }
}
