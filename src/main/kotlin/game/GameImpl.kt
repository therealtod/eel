package eelst.ilike.game

import eelst.ilike.game.variant.Variant

class GameImpl(
    players: Map<PlayerId, GloballyAvailablePlayerInfo>,
    variant: Variant,
    dynamicGloballyAvailableInfo: DynamicGloballyAvailableInfo,
) : BaseGame(
    variant = variant,
    globallyAvailablePlayerInfo = players,
    dynamicGloballyAvailableInfo = dynamicGloballyAvailableInfo,
) {
    override val defaultHandsSize = GameUtils.getHandSize(players.size)
}
