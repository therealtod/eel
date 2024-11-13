package eelst.ilike.game

import eelst.ilike.game.entity.card.HanabiCard

data class TrashPile(
    val cards: List<HanabiCard> = emptyList()
) {
    fun copiesOf(card: HanabiCard): Int {
        return cards.filter { it == card }.size
    }
}
