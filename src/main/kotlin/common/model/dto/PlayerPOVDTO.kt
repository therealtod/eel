package eelst.ilike.utils.model.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming
import eelst.ilike.game.PlayerId

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class PlayerPOVDTO(
    val playerId: PlayerId,
    val players: List<PlayerDTO>
) {
    fun getPlayerDTO(playerId: PlayerId): PlayerDTO {
        return players.find { it.playerId == playerId } ?: throw IllegalArgumentException(
            "Can't find a player with ID: $playerId"
        )
    }
}
