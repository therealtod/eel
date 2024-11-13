package eelst.ilike.game.variant

import eelst.ilike.game.entity.suite.Suite


sealed class Variant(
    val name: String,
    val suites: Set<Suite>,
) {
    companion object {
        val registeredVariants = mapOf(
            NoVariant.name to NoVariant,
        )
        fun getVariantByName(variantName: String): Variant {
            return registeredVariants[variantName]
                ?: throw IllegalArgumentException("No registered variant with name $variantName")
        }
    }
}
