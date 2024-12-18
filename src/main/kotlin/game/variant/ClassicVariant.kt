package eelst.ilike.game.variant

import eelst.ilike.common.model.metadata.VariantMetadata
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suite

class ClassicVariant(
    private val variantMetadata: VariantMetadata,
    suits: Set<Suite>,
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