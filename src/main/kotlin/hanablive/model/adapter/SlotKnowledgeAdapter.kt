package eelst.ilike.hanablive.model.adapter

import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.engine.player.knowledge.PersonalSlotKnowledge
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

class SlotKnowledgeAdapter(drawActionData: GameDrawActionData)
    : PersonalSlotKnowledge {
    override fun getOwnerId(): PlayerId {
        TODO("Not yet implemented")
    }

    override fun getSlotIndex(): Int {
        TODO("Not yet implemented")
    }
    override fun getPossibleSlotIdentities(): Set<HanabiCard> {
        TODO("Not yet implemented")
    }

    override fun isSlotKnown(): Boolean {
        TODO("Not yet implemented")
    }

    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        TODO()
    }

    override fun getEmpathy(): Set<HanabiCard> {
        TODO("Not yet implemented")
    }
}