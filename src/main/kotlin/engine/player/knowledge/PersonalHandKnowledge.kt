package eelst.ilike.engine.player.knowledge

interface PersonalHandKnowledge {
    fun getHandSize(): Int
    fun getKnowledge(slotIndex: Int): PersonalSlotKnowledge
}
