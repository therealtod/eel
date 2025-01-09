package eelst.ilike.game.factory

import eelst.ilike.common.metadata.SuitMetadata
import eelst.ilike.game.entity.Variant
import eelst.ilike.game.entity.suit.Suit


/**
 * Collection of factory methods to create instances of [Suit]
 */
object SuitFactory {
    fun createSuit(metadata: SuitMetadata): Suit {
        val suitName = metadata.name
        if(!supportedSuits.contains(suitName)) {
            throw UnsupportedOperationException("Unsupported suite: $suitName")
        }
        val suitColors = metadata.clueColors
        require(suitColors.size == 1) {
            "Invalid number of colors touching suit: $suitName"
        }
        return ClassicSuit(
            id = metadata.id,
            name = metadata.name,
            abbreviations = listOf(
                metadata.abbreviation,
                metadata.name,
                metadata.id,
                metadata.abbreviation.lowercase(),
                metadata.id.lowercase(),
            ),
            definingColor = Color.getFromStringFormat(suitColors.first())
        )
    }

    private val supportedSuits = listOf("Red", "Yellow", "Green", "Blue", "Purple", "Teal")
}
