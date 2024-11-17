package eelst.ilike.engine


import eelst.ilike.game.PlayerId

interface PersonalInfo {
    fun getOwnSlotInfo(slotIndex: Int): PersonalSlotInfo
    fun getInfo(playerId: PlayerId): InfoOnTeammate
}
