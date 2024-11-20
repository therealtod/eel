package eelst.ilike.engine.player

import eelst.ilike.engine.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.factory.PlayerFactory
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.GameUtils
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

class OldActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfo,
    personalKnowledge: PersonalKnowledge,
): ConventionsUsingPlayer(
    playerId = playerId,
    playerIndex = playerIndex,
    globallyAvailableInfo = globallyAvailableInfo,
    personalKnowledge = personalKnowledge,
) {
    val playerPOV: PlayerPOV = PlayerFactory.createPlayerPOV(
        playerId = playerId,
        playerIndex = playerIndex,
        globallyAvailableInfo = globallyAvailableInfo,
        ownHand = ownHand,
        personalKnowledge = personalKnowledge,
    )

    val teammates: Set<Teammate> = globallyAvailableInfo.players.filter { it.key != playerId }.map {
        PlayerFactory.createTeammate(
            teammateId = it.key,
            globallyAvailableInfo = globallyAvailableInfo,
            personalKnowledge = personalKnowledge,
            seatsGap = GameUtils.getSeatsGap(
                playerIndex1 = playerIndex,
                playerIndex2 = it.value.playerIndex,
                numberOfPlayers = globallyAvailableInfo.numberOfPlayers
            )

        )
    }.toSet()

    fun getLegalActions(conventionSet: ConventionSet): Set<ConventionalAction> {
        val candidateActions = conventionSet
            .getTechs()
            .flatMap { tech->
                tech.getGameActions(playerPOV).map {
                    ConventionalAction(
                        action = it,
                        tech = tech
                    )
                }
            }
        return getPrunedAction(candidateActions)
    }

    override fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return getOwnSlot(slotIndex).contains(card)
    }

    private fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        return actions.toSet()
    }
}