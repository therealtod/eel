package eelst.ilike.hanablive.model.dto.command

import eelst.ilike.hanablive.model.dto.GameOptions

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
    val options: GameOptions,
    val timed: Boolean,
    val timeBase: Int,
    val timePerTurn: Int,
    val sharedReplay: Boolean,
    val progress: Int,
    val players: List<String>,
    val spectators: List<Spectator>,
    val maxPlayers: Int,
) {

    data class Spectator(
        val name: String,
        val shadowingPlayerIndex: Int,
        val shadowingPlayerUsername: String,
    )
}
