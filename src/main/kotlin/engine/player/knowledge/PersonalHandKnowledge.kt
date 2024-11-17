package eelst.ilike.engine.player.knowledge

interface PersonalHandKnowledge {
    fun getKnowledge(slotIndex: Int): PersonalSlotKnowledge
}
