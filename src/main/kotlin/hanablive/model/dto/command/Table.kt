package eelst.ilike.hanablive.model.dto.command

data class Table(
    val id: Int,
    val name: String,
    val passwordProtected: Boolean,
    val joined: Boolean,
    val numPlayers: Int,
    val startingPlayer: Int,
    val owned: Boolean,
    val running: Boolean,
    val variant: String,
    val options: TableOptions,
    val timed: Boolean,
    val timeBase: Int,
    val timePerTurn: Int,
    val sharedReplay: Boolean,
    val progress: Int,
    val players: List<String>,
    val spectators: List<Spectator>,
    val maxPlayers: Int,
) {
    data class TableOptions(
        val numPlayers: Int,
        val startingPlayer: Int,
        val variantId: Int,
        val variantName: String,
        val timed: Boolean,
        val timeBase: Int,
        val timePerTurn: Int,
        val speedrun: Boolean,
        val cardCycle: Boolean,
        val deckPlays: Boolean,
        val emptyClues: Boolean,
        val oneExtraCard: Boolean,
        val oneLessCard:Boolean,
        val allOrNothing: Boolean,
        val detrimentalCharacters: Boolean
    )

    data class Spectator(
        val name: String,
        val shadowingPlayerIndex: Int,
        val shadowingPlayerUsername: String,
    )
}
