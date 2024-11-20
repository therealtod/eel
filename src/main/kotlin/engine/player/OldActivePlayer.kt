package eelst.ilike.engine.player

import eelst.ilike.engine.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.factory.HandFactory
import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.hand.slot.InterpretedSlot
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class OldActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    personalKnowledge: PersonalKnowledge,
): BasePlayer(
    playerId = playerId,
    playerIndex = playerIndex,
) {
    override val ownHand = HandFactory.createOwnHand(
        handSize = globallyAvailableInfo.handsSize,
        playerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId),
        personalSlotKnowledge = personalKnowledge.getSlotKnowledgeAsSet()
    )

    val teammates: Set<Teammate> = globallyAvailableInfo.players.map {
        PlayerFactory.createTeammate(
            teammateId = it.key,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = KnowledgeFactory.createEmptyPersonalKnowledge(),
            hand = personalKnowledge.getTeammateHand(it.key),
            seatsGap = GameUtils.getSeatsGap(
                playerIndex1 = playerIndex,
                playerIndex2 = it.value.playerIndex,
                numberOfPlayers = globallyAvailableInfo.numberOfPlayers
            )

        )
    }.toSet()

    override val playerPOV = TODO()

    fun getLegalActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet.getTechs().flatMap { it.getActions(playerPOV) }
        return getPrunedAction(candidateActions)
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }

    override fun getSlots(): Set<InterpretedSlot> {
        TODO("Not yet implemented")
    }

    override fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        val slot = ownHand.getSlot(slotIndex)
        return slot.hasKnownIdentity(card)
    }
}