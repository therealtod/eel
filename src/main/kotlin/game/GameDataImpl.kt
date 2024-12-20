package eelst.ilike.game

import eelst.ilike.game.variant.Variant

class GameDataImpl(
    players: Map<PlayerId, PlayerMetadata>,
    variant: Variant,
    dynamicGloballyAvailableInfo: DynamicGloballyAvailableInfo,
) : BaseGameData(
    variant = variant,
    playerMetadata = players,
    dynamicGloballyAvailableInfo = dynamicGloballyAvailableInfo,
) {
    override val defaultHandsSize = GameUtils.getHandSize(players.size)
}
