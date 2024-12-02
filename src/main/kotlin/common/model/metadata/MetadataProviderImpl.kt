package eelst.ilike.common.model.metadata

import com.fasterxml.jackson.module.kotlin.readValue
import eelst.ilike.utils.Utils

data object MetadataProviderImpl : MetadataProvider {
    private val variantsMetadata: List<VariantMetadata> by lazy {
        Utils.jsonObjectMapper.readValue(
            Utils.getResourceFileContentAsString(
                "variants_metadata_mirror.json"
            )
        )
    }

    private val suitsMetadata: List<SuiteMetadata> by lazy {
        Utils.jsonObjectMapper.readValue(
            Utils.getResourceFileContentAsString(
                "suits_metadata_mirror.json"
            )
        )
    }

    override fun getVariantMetadata(variantName: String): VariantMetadata {
        return variantsMetadata.first { it.name == variantName }
    }

    override fun getSuiteMetadata(suiteId: String): SuiteMetadata {
        return suitsMetadata.first { it.name.equals(suiteId,  ignoreCase = true) }
    }
}
