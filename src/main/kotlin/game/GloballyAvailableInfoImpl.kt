package eelst.ilike.game

import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.utils.Utils

class GloballyAvailableInfoImpl(
    override val suits: Set<Suite>,
    override val variant: Variant,
    override val players: Map<PlayerId, GloballyAvailablePlayerInfo>,
    dynamicGloballyAvailableInfo: DynamicGloballyAvailableInfo,
) : BaseGloballyAvailableInfo(
    playersIds = players.keys,
    globallyAvailablePlayerInfo = players.values.toSet(),
    dynamicGloballyAvailableInfo = dynamicGloballyAvailableInfo,
) {
    override val defaultHandsSize = Utils.getHandSize(players.size)
    override val numberOfPlayers = players.size
}
