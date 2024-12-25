package eelst.ilike.game.variant

import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suit

class ClassicVariant(
    private val variantMetadata: VariantMetadata,
    suits: Set<Suit>,
): Variant(
    id = variantMetadata.newID,
    name = variantMetadata.name,
    suits = suits,
) {
    override fun getCluableRanks(): Set<Rank> {
        return setOf(
            Rank.ONE, Rank.TWO, Rank.THREE, Rank.FOUR, Rank.FIVE
        )
    }
}