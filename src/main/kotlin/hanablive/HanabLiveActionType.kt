package eelst.ilike.hanablive

enum class HanabLiveActionType(val jsonValue: Int) {
    PLAY(0),
    DISCARD(1),
    COLOR_CLUE(2),
    RANK_CLUE(3),
    END_GAME(4);
}
