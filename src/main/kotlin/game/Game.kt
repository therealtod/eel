package eelst.ilike.game

import eelst.ilike.game.entity.GameEvent
import eelst.ilike.game.entity.Player
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.variant.Variant

interface Game {
    fun getPlayer(playerId: PlayerId): Player
    fun getEvents(): List<GameEvent>
    fun getVariant(): Variant
    fun isCritical(card: HanabiCard): Boolean
}
