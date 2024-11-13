package eelst.ilike.game


data class GloballyAvailablePlayerInfo(
    val playerId: PlayerId,
    val playerIndex: Int,
    val hand: Set<GloballyAvailableSlotInfo>
)
