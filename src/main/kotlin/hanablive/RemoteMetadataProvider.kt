package eelst.ilike.hanablive

import eelst.ilike.common.model.metadata.*
import eelst.ilike.hanablive.client.MetadataClient
import kotlinx.coroutines.runBlocking

object RemoteMetadataProvider: MetadataProvider {
    private val metadataClient = MetadataClient
    
    override fun getSuiteMetadata(suiteId: String): SuitMetadata {
        TODO("Not yet implemented")
    }

    override fun getVariantMetadata(variantName: String): VariantMetadata {
        val variantsMetadata = runBlocking { metadataClient.getVariantsMetadata() }
        return variantsMetadata.find { it.name == variantName }
            ?: throw IllegalStateException("Could not find metadata for game variant: ${variantName}")
    }

    override fun getSuitsMetadata(suiteIds: Collection<String>): Map<String, SuitMetadata> {
        val metadata = runBlocking { metadataClient.getSuitsMetadata() }
        return metadata
            .associateBy { it.name }
            .filterKeys { suiteIds.contains(it) }
    }
}