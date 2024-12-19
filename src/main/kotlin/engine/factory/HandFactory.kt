package eelst.ilike.engine.factory

import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand

object HandFactory {
    /*
    fun createPlayerHand(
        playerId: PlayerId,
        handSize: Int,
        personalKnowledge: PlayerPersonalKnowledge,
        globallyAvailableSlotInfo: Collection<GloballyAvailableSlotInfo>
        ): Hand {
        return if (personalKnowledge.canSee(playerId)) {
           return personalKnowledge.getVisibleHand(playerId)
        } else {
            createOwnHand(
                playerId = playerId,
                handSize = handSize,
                playerPersonalKnowledge = personalKnowledge,
                globallyAvailableSlotInfo = globallyAvailableSlotInfo,
            )
        }
    }

     */
/*
    fun createOwnHand(
        playerId: PlayerId,
        handSize: Int,
        playerPersonalKnowledge: PlayerPersonalKnowledge,
        globallyAvailableSlotInfo: Collection<GloballyAvailableSlotInfo>,
    ): Hand {
        val slots = (1..handSize).map {
            SlotFactory.createSlot(
                globalInfo = globallyAvailableSlotInfo.elementAt(it - 1),
                knowledge = playerPersonalKnowledge.getOwnHandKnowledge(playerId).getKnowledge(it)
            )
        }
        return  SimpleHand(
            ownerId = playerId,
            slots = slots.toSet(),
        )
    }

 */

    fun createHand(
        playerId: PlayerId,
        personalKnowledge: PlayerPersonalKnowledge,
        hand: Hand
    ): Hand {
        TODO()
    }


}