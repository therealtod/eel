package eelst.ilike.hanablive.model.adapter

import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

class SlotKnowledgeAdapter(drawActionData: GameDrawActionData)
    : PersonalSlotKnowledge {
    override fun getImpliedIdentities(): Set<HanabiCard> {
        TODO("Not yet implemented")
    }
}
