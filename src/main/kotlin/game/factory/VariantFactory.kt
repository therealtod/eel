package eelst.ilike.game.factory

import eelst.ilike.common.model.metadata.LocalMirrorMetadataProvider
import eelst.ilike.common.model.metadata.SuitMetadata
import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.ClassicVariant
import eelst.ilike.game.variant.Variant

object VariantFactory {
    fun createVariant(metadata: VariantMetadata): Variant {
        val variantName = metadata.name
        if (!supportedVariants.contains(variantName)) {
            throw UnsupportedOperationException("Variant currently not supported: $variantName")
        }
        val suitsMetadata = metadataProvider.getSuitsMetadata(metadata.suits)
        val suits = suitsMetadata.map { SuitFactory.createSuit(it.value) }
        return ClassicVariant(
            variantMetadata = metadata,
            suits = suits.toSet()
        )
    }

    private val supportedVariants = listOf("No Variant", "6 Suits")
    private val metadataProvider = LocalMirrorMetadataProvider
}
