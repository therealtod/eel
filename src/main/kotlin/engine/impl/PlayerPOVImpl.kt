package eelst.ilike.engine.impl

import eelst.ilike.engine.*
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.Slot
import eelst.ilike.game.entity.card.HanabiCard

class PlayerPOVImpl(
    override val playerId: PlayerId,
    override val globallyAvailableInfo: GloballyAvailableInfo,
    personalInfo: PersonalInfo = PersonalInfoImpl(),
): PlayerPOV {
    override val hand: OwnHand
    override val teammates: Set<Teammate>

    init {
        val thisPlayerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val mySlots = thisPlayerGlobalInfo.hand.map {
            OwnSlot(
                globalInfo = it,
                impliedIdentities = personalInfo.getOwnSlotInfo(it.index).impliedIdentities
            )
        }
        hand = OwnHand(
            slots = mySlots.toSet(),
            playerPOV = this,
        )
        val numberOfPlayers = globallyAvailableInfo.players.size
        val myPlayerIndex = thisPlayerGlobalInfo.playerIndex
        teammates = globallyAvailableInfo.players.filterKeys { it != playerId }.values.map { player->
            Teammate(
                playerId = player.playerId,
                seatsGap = (numberOfPlayers- myPlayerIndex + player.playerIndex).mod(numberOfPlayers),
                playerPOV = PlayerPOVImpl(
                    playerId = player.playerId,
                    globallyAvailableInfo = globallyAvailableInfo,
                ),
                hand = personalInfo.getTeammateHand(player.playerId)
            )
        }.toSet()
    }

    override fun getVisibleCards(): List<HanabiCard> {
        return globallyAvailableInfo.cardsOnStacks +
                globallyAvailableInfo.trashPile.cards +
                hand.getKnownCards() +
                teammates.flatMap { teammate-> teammate.hand.getCards() }
    }

    override fun allCardsAreKnown(cards: Set<HanabiCard>): Boolean {
        TODO("Not yet implemented")
    }

    override fun getActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(this) }
        return getPrunedAction(candidateActions)
    }

    override fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }

    override fun getKnownPlayableSlots(): Set<Slot> {
        return hand.getKnownSlots().filter { globallyAvailableInfo.isImmediatelyPlayable(it.getCard()) }.toSet()
    }

}