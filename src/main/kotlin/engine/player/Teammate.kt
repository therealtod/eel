package eelst.ilike.engine.player

import eelst.ilike.engine.BasePlayer
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.engine.hand.slot.VisibleSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class Teammate(
    val hand: TeammateHand,
    val seatsGap: Int,
    playerId: PlayerId,
    playerIndex: Int,
    ownHand: OwnHand,
    globallyAvailableInfo: GloballyAvailableInfo,
    personalKnowledge: PersonalKnowledge,
) : BasePlayer(
    playerId = playerId,
    playerIndex = playerIndex,
    ownHand = ownHand,
) {
    override val playerPOV = PlayerFactory.createPlayerPOV(
        globallyAvailableInfo = globallyAvailableInfo,
        ownHand = ownHand,
        personalKnowledge = personalKnowledge
    )

    override val ownHand = playerPOV.ownHand

    fun playsBefore(otherTeammate: Teammate): Boolean {
        return seatsGap < otherTeammate.seatsGap
    }

    override fun getCardAtSlot(slotIndex: Int): HanabiCard {
        return hand.getSlot(slotIndex).card
    }

    fun getSlot(slotIndex: Int): VisibleSlot {
        return hand.getSlot(slotIndex)
    }

    override fun buildPlayerPOV(
        globallyAvailableInfo: GloballyAvailableInfo,
        ownHand: OwnHand,
        players: Set<BasePlayer>
    ): PlayerPOV {
        TODO("Not yet implemented")
    }
}
