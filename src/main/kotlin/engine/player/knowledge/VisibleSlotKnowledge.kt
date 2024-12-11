package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.GloballyAvailableSlotInfo
import eelst.ilike.game.entity.card.HanabiCard

data class VisibleSlotKnowledge(
    val impliedIdentities: Set<HanabiCard>,
    val slotIdentity: HanabiCard,
): PersonalSlotKnowledge {
    override fun plus(knowledge: PersonalSlotKnowledge): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}