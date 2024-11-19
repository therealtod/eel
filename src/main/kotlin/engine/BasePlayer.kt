package eelst.ilike.engine

import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.slot.KnownSlot
import eelst.ilike.engine.hand.slot.OwnSlot
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard

abstract class BasePlayer(
    override val playerId: PlayerId,
    override val playerIndex: Int,
) : Player<OwnHand> {
    override val ownHand: OwnHand
        get() = playerPOV.ownHand
    abstract val playerPOV: PlayerPOV

    fun getOwnKnownCards(): List<HanabiCard> {
        return getOwnKnownSlots().map { it.card }
    }

    fun getOwnKnownSlots(): Set<KnownSlot> {
        return ownHand.getKnownSlots()
    }

    fun getSlotFromPlayerPOV(slotIndex: Int): OwnSlot {
        return ownHand.getSlot(slotIndex)
    }

    fun knows(slotIndex: Int): Boolean {
        return ownHand.getSlot(slotIndex).isKnown()
    }

    fun forEachTeammate(action: (teammate: BasePlayer) -> Unit) {
        return playerPOV.forEachTeammate(action)
    }

    abstract fun buildPlayerPOV(
        globallyAvailableInfo: GloballyAvailableInfo,
        ownHand: OwnHand,
        players: Set<BasePlayer>,
    ): PlayerPOV
}
