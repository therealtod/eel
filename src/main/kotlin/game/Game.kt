package eelst.ilike.game

import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.Player
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.slot.Slot
import eelst.ilike.game.entity.suit.Suit
import eelst.ilike.game.entity.suit.SuitId


interface Game {
    val playingStacks: Map<SuitId, PlayingStack>
    val suits: Set<Suit>
    val trashPile: TrashPile
    val strikes: Int
    val efficiency: Float
    val pace: Int
    val variant: Variant
    val clueTokens: Int
    val players: Map<PlayerId, Player>
    val defaultHandsSize: Int
    val numberOfPlayers: Int
    val score: Int
    fun getCardsOnStacks(): List<HanabiCard>
    fun getStackForCard(card: HanabiCard): PlayingStack
    fun isAlreadyPlayed(card: HanabiCard): Boolean
    fun isCritical(
        card: HanabiCard,
    ): Boolean

    /**
     * @return n as in "the slot is n-from playable"
     */
    fun getGlobalAwayValue(card: HanabiCard): Int
    fun isImmediatelyPlayable(card: HanabiCard): Boolean
    fun getPlayer(playerId: PlayerId): Player
    fun getPlayer(playerIndex: Int): Player
    fun getAvailableClueValues(): Set<ClueValue>
    fun getAfter(drawAction: DrawAction): Game
    fun getAfter(playAction: PlayAction, playedCard: HanabiCard): Game
    fun getAfter(discardAction: DiscardAction, discardedCard: HanabiCard): Game
    fun getAfter(clueAction: ClueAction): Game
}
