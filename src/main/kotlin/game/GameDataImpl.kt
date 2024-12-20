package eelst.ilike.game

import eelst.ilike.game.variant.Variant

class GameDataImpl(
    players: Map<PlayerId, PlayerMetadata>,
    variant: Variant,
    dynamicGameData: DynamicGameData,
) : BaseGameData(
    variant = variant,
    playerMetadata = players,
    dynamicGameData = dynamicGameData,
) {
    override val defaultHandsSize = GameUtils.getHandSize(players.size)
}
