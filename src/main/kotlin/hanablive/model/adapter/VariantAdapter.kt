package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.variant.Variant
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata

class VariantAdapter(
    variantMetadata: VariantMetadata,
    suiteMetadata: List<SuiteMetadata>,
)
    : Variant(
        name = variantMetadata.name,
        suits = variantMetadata.suits.map {
            SuiteAdapter(suiteMetadata.find { it.id == it })
        }
    ) {

}