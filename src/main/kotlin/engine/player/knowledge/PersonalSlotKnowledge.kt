package eelst.ilike.engine.player.knowledge

import eelst.ilike.game.entity.card.HanabiCard

interface PersonalSlotKnowledge: PersonalKnowledge {
    fun getImpliedIdentities(): Set<HanabiCard>
}
