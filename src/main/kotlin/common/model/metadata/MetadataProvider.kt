package eelst.ilike.common.model.metadata

interface MetadataProvider {
    fun getVariantMetadata(variantName: String): VariantMetadata
    fun getSuiteMetadata(suiteId: String): SuitMetadata
    fun getSuitsMetadata(suiteIds: Collection<String>): Map<String, SuitMetadata>
}
