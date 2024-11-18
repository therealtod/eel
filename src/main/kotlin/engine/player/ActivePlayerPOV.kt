package eelst.ilike.engine.player

import eelst.ilike.engine.EngineCommon
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.OwnHand
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class ActivePlayerPOV(
    playerId: PlayerId,
    playerIndex: Int,
    hand: OwnHand,
    globallyAvailableInfo: GloballyAvailableInfo,
    val teammates: Set<Teammate>,
) : PlayerPOV(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    ownHand = hand
) {
    fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(this) }
        return EngineCommon.getPrunedAction(candidateActions)
    }

    fun teamKnowsAllCards(cards: Set<HanabiCard>): Boolean {
        val allKnownCards = teammates
            .flatMap { it.getOwnKnownCards() } +
                getOwnKnownCards()
        return allKnownCards.containsAll(cards)
    }
}