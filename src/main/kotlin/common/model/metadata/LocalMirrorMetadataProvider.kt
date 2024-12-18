package eelst.ilike.common.model.metadata

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.utils.Utils

data object LocalMirrorMetadataProvider : MetadataProvider {
    private val variantsMetadata: List<EelVariantMetadata> by lazy {
        Utils.jsonObjectMapper.readValue(
            Utils.getResourceFileContentAsString(
                "variants_metadata_mirror.json"
            )
        )
    }

    private val suitsMetadata: List<EelSuiteMetadata> by lazy {
        Utils.jsonObjectMapper.readValue(
            Utils.getResourceFileContentAsString(
                "suits_metadata_mirror.json"
            )
        )
    }

    override fun getVariantMetadata(variantName: String): EelVariantMetadata {
        return variantsMetadata.first { it.name == variantName }
    }

    override fun getSuiteMetadata(suiteId: String): EelSuiteMetadata {
        return suitsMetadata.first { it.name.equals(suiteId,  ignoreCase = true) }
    }

    override fun getSuitsMetadata(suiteIds: Collection<String>): Map<String, SuitMetadata> {
        return suitsMetadata
            .associateBy { it.name }
            .filterKeys { suiteIds.contains(it) }
    }
}
