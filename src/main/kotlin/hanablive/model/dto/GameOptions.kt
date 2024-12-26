package eelst.ilike.hanablive.model.dto

data class GameOptions(
    val maxPlayers: Int? = null,
    val numPlayers: Int,
    val startingPlayer: Int,
    val variantID: Int,
    val variantName: String,
    val tableName: String? = null,
    val timed: Boolean,
    val timeBase: Int,
    val timePerTurn: Int,
    val speedrun: Boolean,
    val cardCycle: Boolean,
    val deckPlays: Boolean,
    val emptyClues: Boolean,
    val oneExtraCard: Boolean,
    val oneLessCard: Boolean,
    val allOrNothing: Boolean,
    val detrimentalCharacters: Boolean
)
