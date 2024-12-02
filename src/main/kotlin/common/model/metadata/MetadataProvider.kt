package eelst.ilike.common.model.metadata

interface MetadataProvider {
    fun getVariantMetadata(variantName: String): VariantMetadata
    fun getSuiteMetadata(suiteId: String): SuiteMetadata
}
