package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.BaseGloballyAvailableInfo
import eelst.ilike.game.DynamicGloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata

class GloballyAvailableInfoAdapter(
    playerIds: Set<PlayerId>,
    variantMetadata: VariantMetadata,
    suitsMetadata: Collection<SuiteMetadata>,
) : BaseGloballyAvailableInfo(
    playersIds = playerIds,
    globallyAvailablePlayerInfo = TODO(),
    dynamicGloballyAvailableInfo = DynamicGloballyAvailableInfo(
        playingStacks = TODO(),
        trashPile = TrashPile(),
        strikes = 0,
        clueTokens = 8,
        pace = TODO(),
        efficiency = TODO(),
    )
) {
    override val efficiency = TODO()

    override val handsSize = TODO()

    override val numberOfPlayers = TODO()

    override val pace = TODO()

    override val players = TODO()

    override val variant = VariantAdapter(
        variantMetadata = variantMetadata,
        suitsMetadata = suitsMetadata,
    )

    override val suits = variant.suits
}
