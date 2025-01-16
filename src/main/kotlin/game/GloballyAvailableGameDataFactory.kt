package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.entity.variant.VariantFactory

/**
 * Collection of factory methods used to create instances of [GloballyAvailableGameData]
 */
object GloballyAvailableGameDataFactory {
    fun createGloballyAvailableGameData(
        variantMetadata: VariantMetadata,
        suitsMetadata: Map<SuitId, SuitMetadata>,
        playersMetadata: List<PlayerMetadata>,
    ): GloballyAvailableGameData {
        val variant = VariantFactory.createVariant(variantMetadata, suitsMetadata)
        val suits = variant.getSuits()
        return GloballyAvailableGameData(
            variant = variant,
            playingStacks = suits.associate { it.id to PlayingStack(emptyList(), it) },
            trashPile = TrashPile(),
            strikes = GameConstants.INITIAL_STRIKE_TOKENS_COUNT,
            clueTokens = GameConstants.MAX_CLUE_TOKENS_COUNT,
            amountOfCardsPlayed = 0,
            amountOfCardsDiscarded = 0,
            playersMetadata = playersMetadata,
        )
    }
}
