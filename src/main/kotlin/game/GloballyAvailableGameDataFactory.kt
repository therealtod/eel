package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.suit.SuitId
import eelst.ilike.game.entity.suit.SuitMetadata
import eelst.ilike.game.entity.variant.Variant
import eelst.ilike.game.entity.variant.VariantMetadata
import eelst.ilike.game.entity.variant.VariantFactory

/**
 * Collection of factory methods used to create instances of [GloballyAvailableGameData]
 */
object GloballyAvailableGameDataFactory {
    fun createGloballyAvailableGameData(
        variant: Variant,
        playersMetadata: List<PlayerMetadata>,
    ): GloballyAvailableGameData {
        val suits = variant.getSuits()
        val players = playersMetadata.map {
            Player(
                playerMetadata = it,
                hand = emptyList(),
            )
        }
        return GloballyAvailableGameData(
            variant = variant,
            playingStacks = suits.associate { it.id to PlayingStack(emptyList(), it) },
            trashPile = TrashPile(),
            strikes = GameConstants.INITIAL_STRIKE_TOKENS_COUNT,
            clueTokens = GameConstants.MAX_CLUE_TOKENS_COUNT,
            players = players.toMutableList(),
        )
    }
}
