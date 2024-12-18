package eelst.ilike.game.factory

import eelst.ilike.common.model.metadata.SuitMetadata
import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.ClassicVariant
import eelst.ilike.game.variant.Variant

object VariantFactory {
    fun createVariant(metadata: VariantMetadata, suits: Set<Suite>): Variant {
        val variantName = metadata.name
        if (!supportedVariants.contains(variantName)) {
            throw UnsupportedOperationException("Variant currently not supported: $variantName")
        }
        return ClassicVariant(
            variantMetadata = metadata,
            suits = suits
        )
    }

    private val supportedVariants = listOf("No Variant", "6 Suits")
}
