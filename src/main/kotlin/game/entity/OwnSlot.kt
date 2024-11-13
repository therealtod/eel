package eelst.ilike.game.entity

/*
open class OwnSlot(
    index: Int,
    notes: List<Note> = emptyList(),
    private val impliedIdentities: Set<HanabiCard> = emptySet(),
): Slot(
    index = index,
    notes = notes,
)  {
    fun isKnownBy(playerPOV: PlayerPOV): Boolean {
        return getPossibleIdentities(playerPOV).size == 1
    }

    override fun isClued(playerPOV: PlayerPOV): Boolean {
        val playerGlobalInfo = playerPOV.globallyAvailableInfo.getPlayerInfo(playerPOV.playerId)
      return playerGlobalInfo.getSlotInfo(index).positiveClues.isNotEmpty() ||
              impliedIdentities.isNotEmpty()
    }

    override fun getPossibleIdentities(): Set<HanabiCard> {
        return impliedIdentities.ifEmpty {
            playerPOV.globallyAvailableInfo.getPlayerInfo(playerPOV.playerId).getSlotInfo(index).getEmpathy(playerPOV)
        }
    }
}

 */
