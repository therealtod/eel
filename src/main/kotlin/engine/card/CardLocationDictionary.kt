package engine.card


class CardLocationDictionary(private val dictionary: MutableList<List<Int>> = mutableListOf()) {
    fun getPlayerSlotIndex(playerIndex: Int, cardOrder: Int): Int {
        return dictionary[playerIndex].indexOf(cardOrder) + 1
    }

    fun addCard(playerIndex: Int, cardOrder: Int): CardLocationDictionary {
        dictionary[playerIndex] = listOf(cardOrder) + dictionary[playerIndex]
        return this
    }

    fun removeCard(playerIndex: Int, cardOrder: Int): CardLocationDictionary {
        dictionary[playerIndex] = dictionary[playerIndex].minus(cardOrder)
        return this
    }
}
