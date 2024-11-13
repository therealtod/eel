package eelst.ilike.game

import eelst.ilike.engine.PlayerPOV
import eelst.ilike.game.action.Clue
import eelst.ilike.game.entity.card.HanabiCard

interface Slot{
    val index: Int
    val positiveClues: List<Clue>
    val negativeClues: List<Clue>
    fun isTouched(): Boolean
    fun getEmpathy(playerPOV: PlayerPOV): Set<HanabiCard>
    abstract fun getCard(): HanabiCard
}
 /*
override fun isTouched(): Boolean {
    return positiveClues.isNotEmpty()
}

override fun isClued(playerPOV: PlayerPOV): Boolean{
    TODO()
}

 {

}

override fun getCard(): HanabiCard {
    TODO("Not yet implemented")
}

override fun isKnown(): Boolean {
    return impliedIdentities.size == 1
}

 = true

override fun getPossibleIdentities(): Set<HanabiCard> {
    return impliedIdentities
}

  */