package eelst.ilike.game.variant

import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suite


abstract class Variant(
    val id: String,
    val name: String,
    val suits: Set<Suite>,
) {
    abstract fun getCluableRanks(): Set<Rank>

    abstract fun getCluableColors(): Set<Color>

    companion object {
        private val registeredVariants = mapOf(
            NoVariant.name to NoVariant,
        )

        fun getVariantByName(variantName: String): Variant {
            return registeredVariants[variantName]
                ?: throw IllegalArgumentException("No registered variant with name $variantName")
        }
    }
}
