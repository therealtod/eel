package eelst.ilike.game

import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.Utils

class GloballyAvailableInfoImpl(
    players: Map<PlayerId, GloballyAvailablePlayerInfo>,
    variant: Variant,
    dynamicGloballyAvailableInfo: DynamicGloballyAvailableInfo,
) : BaseGloballyAvailableInfo(
    variant = variant,
    globallyAvailablePlayerInfo = players,
    dynamicGloballyAvailableInfo = dynamicGloballyAvailableInfo,
) {
    override val defaultHandsSize = GameUtils.getHandSize(players.size)
}
