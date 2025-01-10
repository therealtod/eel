package eelst.ilike.game.factory

import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.Variant
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.variant.ClassicVariant


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
            id = metadata.newID,
            name = metadata.name,
            suits = suits
        )
    }

    private val supportedVariants = listOf("No Variant", "6 Suits")
}
