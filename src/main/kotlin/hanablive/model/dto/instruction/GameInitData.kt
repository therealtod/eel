package eelst.ilike.hanablive.model.dto.command

import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.model.dto.GameOptions
import java.time.ZonedDateTime

data class GameInitData(
    val tableID: Int,
    val playerNames: List<PlayerId>,
    val ourPlayerIndex: Int,
    val spectating: Boolean,
    val shadowing: Boolean,
    val replay: Boolean,
    val databaseID: Int,
    val hasCustomSeed: Boolean,
    val seed: String,
    val datetimeStarted: ZonedDateTime,
    val datetimeFinished: ZonedDateTime,
    val options: GameOptions,
    val characterAssignments: List<Any>,
    val characterMetadata: List<Any>,
    val sharedReplay: Boolean,
    val sharedReplayLeader: PlayerId,
    val sharedReplaySegment: Int,
    val sharedReplayEffMod: Int,
    val paused: Boolean,
    val pausePlayerIndex: Int,
    val pauseQueued: Boolean
)
