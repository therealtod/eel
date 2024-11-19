package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.TeammateHand
import eelst.ilike.game.PlayerId

class PersonalKnowledgeImpl(
    private val slotKnowledge: Set<PersonalSlotKnowledge>,
    private val teammatesHands: Map<PlayerId, TeammateHand>
) : PersonalKnowledge {
    override fun getKnowledgeAboutOwnSlot(slotIndex: Int): PersonalSlotKnowledge {
        return slotKnowledge.elementAt(slotIndex - 1)
    }

    override fun getTeammateHand(teammatePlayerId: PlayerId): TeammateHand {
        return teammatesHands[teammatePlayerId]
            ?: throw IllegalArgumentException("The hand of the player with ID $teammatePlayerId is unknown")
    }
}