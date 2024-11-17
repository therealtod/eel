package eelst.ilike.engine.impl

import eelst.ilike.engine.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.Player
import eelst.ilike.game.PlayerId

class ActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    hand: OwnHand,
    globallyAvailableInfo: GloballyAvailableInfo,
    pov: PlayerPOV,
): Player(
    playerId = playerId,
    playerIndex = playerIndex,
    hand = hand,
    globallyAvailableInfo = globallyAvailableInfo,
    playerPOV = pov
) {
}