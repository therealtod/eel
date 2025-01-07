package hanablive.entity.dto

import eelst.ilike.hanablive.entity.Table
import eelst.ilike.hanablive.entity.dto.GameOptions

data class Table(
    override val id: Int,
    override val name: String,
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
    override val players: List<String>,
    val spectators: List<Spectator>,
    val maxPlayers: Int,
): Table {
    data class Spectator(
        val name: String,
        val shadowingPlayerIndex: Int,
        val shadowingPlayerUsername: String,
    )
}
