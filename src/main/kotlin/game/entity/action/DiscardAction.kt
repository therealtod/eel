package eelst.ilike.game.entity.action

import eelst.ilike.engine.card.InGameCard
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.Slot


data class DiscardAction(
    val playerMetadata: PlayerMetadata,
    val slotIndex: Int,
) : GameAction(playerMetadata, GameActionType.DISCARD)
