package hanablive.metadata

import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.suit.SuitId

/**
 * Retriever of metadata for various game entities
 */
interface MetadataProvider {
    /**
     * @return the [VariantMetadata] associated with the given [variantName]
     */
    suspend fun getVariantMetadata(variantName: String): VariantMetadata

    /**
     * @return the [SuitMetadata] associated with the given [suiteId]
     */
    suspend fun getSuitMetadata(suitId: String): SuitMetadata

    /**
     * @return a [Map] associating a [SuitId] to the corresponding [SuitMetadata]
     */
    suspend fun getSuitsMetadata(suitIds: Collection<String>): Map<SuitId, SuitMetadata>
}