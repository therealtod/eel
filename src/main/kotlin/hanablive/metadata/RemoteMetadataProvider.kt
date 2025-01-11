package hanablive.metadata

import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.hanablive.client.MetadataClient
import kotlinx.coroutines.runBlocking

object RemoteMetadataProvider : MetadataProvider {
    private val metadataClient = MetadataClient

    override suspend fun getSuitMetadata(suitId: String): SuitMetadata {
        val metadata = runBlocking { metadataClient.getSuitsMetadata() }
        return metadata.find { it.name == suitId } ?: throw NoSuchElementException(
            "Could not retrieve the metadata for the given $suitId"
        )
    }

    override suspend fun getVariantMetadata(variantName: String): VariantMetadata {
        val variantsMetadata = runBlocking { metadataClient.getVariantsMetadata() }
        return variantsMetadata.find { it.name == variantName }
            ?: throw IllegalStateException("Could not find metadata for game variant: $variantName")
    }

    override suspend fun getSuitsMetadata(suitIds: Collection<SuitId>): Map<SuitId, SuitMetadata> {
        val metadata = runBlocking { metadataClient.getSuitsMetadata() }
        val requestedMetadata = metadata
            .associateBy { it.name }
            .filterKeys { suitIds.contains(it) }
        if (suitIds.any { !requestedMetadata.keys.contains(it) }) {
            throw NoSuchElementException(
                "Could not retrieve the metadata for the requested suit ids: $suitIds"
            )
        }
        return requestedMetadata
    }
}
