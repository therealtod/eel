package eelst.ilike.engine.player.knowledge

interface Knowledge {
    fun getUpdatedWith(knowledge: Knowledge): Knowledge
}
