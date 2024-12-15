package eelst.ilike.game

import eelst.ilike.game.entity.*
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant

interface GloballyAvailableInfo {
    val playingStacks: Map<SuiteId, PlayingStack>
    val suits: Set<Suite>
    val trashPile: TrashPile
    val strikes: Int
    val efficiency: Float
    val pace: Int
    val variant: Variant
    val clueTokens: Int
    val players: Map<PlayerId, GloballyAvailablePlayerInfo>
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
    fun getPlayerInfo(playerId: PlayerId): GloballyAvailablePlayerInfo
    fun getPlayerInfo(playerIndex: Int): GloballyAvailablePlayerInfo
    fun getAvailableClueValues(): Set<ClueValue>
}
