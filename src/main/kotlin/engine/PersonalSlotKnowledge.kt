package eelst.ilike.engine

import eelst.ilike.game.entity.card.HanabiCard

interface PersonalSlotKnowledge {
    fun getPossibleSlotIdentities(): Set<HanabiCard>
    fun isClued(): Boolean
}
