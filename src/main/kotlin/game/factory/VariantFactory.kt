package eelst.ilike.game.factory

import common.metadata.VariantMetadata
import eelst.ilike.common.metadata.SuitMetadata
import eelst.ilike.game.entity.Variant
import eelst.ilike.game.entity.suit.SuitId


/**
 * Collection of factory methods to create instances of [Variant]
 */
object VariantFactory {
    fun createVariant(metadata: VariantMetadata, suitsMetadata: Map<SuitId, SuitMetadata>): Variant {
        val variantName = metadata.name
        if (!supportedVariants.contains(variantName)) {
            throw UnsupportedOperationException("Variant currently not supported: $variantName")
        }
        val suits = suitsMetadata.map { SuitFactory.createSuit(it.value) }
        return ClassicVariant(
            variantMetadata = metadata,
            suits = suits.toSet()
        )
    }

    private val supportedVariants = listOf("No Variant", "6 Suits")
}
