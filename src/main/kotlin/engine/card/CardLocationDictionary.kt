package engine.card


class CardLocationDictionary(private val dictionary: List<List<Int>> = emptyList()) {
    fun getPlayerSlotIndex(playerIndex: Int, cardOrder: Int): Int {
        return dictionary[playerIndex].indexOf(cardOrder) + 1
    }

    fun addCard(playerIndex: Int, cardOrder: Int): CardLocationDictionary {
        val updatedDictionary = dictionary.toMutableList()
        updatedDictionary[playerIndex] = listOf(cardOrder) + dictionary[playerIndex]
        return CardLocationDictionary(updatedDictionary)
    }

    fun removeCard(playerIndex: Int, cardOrder: Int): CardLocationDictionary {
        val updatedDictionary = dictionary.toMutableList()
        updatedDictionary[playerIndex] = dictionary[playerIndex].minus(cardOrder)
        return CardLocationDictionary(updatedDictionary)
    }
}
