package eelst.ilike.hanablive

import eelst.ilike.hanablive.model.dto.command.GameActionType

object HanabLiveConstants {
    const val HOSTNAME = "hanab.live"
    const val METADATA_PROVIDER_HOSTNAME = "raw.githubusercontent.com"
    const val VARIANTS_METADATA_PATH = "/Hanabi-Live/hanabi-live/main/packages/game/src/json/variants.json"
    const val SUITE_METADATA_PATH = "/Hanabi-Live/hanabi-live/main/packages/game/src/json/suits.json"
    const val HANAB_LIVE_HTTP_PORT = 443
    const val LOGIN_PATH = "/login"
    const val WEBSOCKET_PATH = "/ws"
    const val COOKIE_NAME = "set-cookie"
    const val COLOR_CLUE_TYPE = 2
    const val RANK_CLUE_TYPE = 3
    val PLAYER_ACTIONS = listOf(GameActionType.PLAY, GameActionType.DISCARD, GameActionType.CLUE)
}
