package eelst.ilike.engine.factory

import eelst.ilike.engine.OwnSlot
import eelst.ilike.engine.PersonalInfo
import eelst.ilike.engine.Teammate
import eelst.ilike.engine.impl.*
import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard

object PlayerFactory {
    fun createActivePlayer(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        personalInfo: PersonalInfo,
    ): ActivePlayer {
        val thisPlayerGlobalInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val numberOfPlayers = globallyAvailableInfo.players.size
        val activePlayerIndex = thisPlayerGlobalInfo.playerIndex
        val handMap = globallyAvailableInfo.players.filterKeys { it != playerId }.mapValues { entry ->
            TeammateHand(
                entry.value.hand.map {
                    personalInfo.getInfo(entry.key).getSlot(it.index, it)
                }.toSet()
            )
        }
        val cardsOnTeammatesHandsMap = handMap.mapValues {
            it.value.getCards()
        }
        val cardsOnTeammatesHands = cardsOnTeammatesHandsMap.values.flatten()

        val activePlayerSlots = thisPlayerGlobalInfo.hand.map {
            OwnSlot(
                globalInfo = it,
                impliedIdentities = personalInfo.getOwnSlotInfo(it.index).impliedIdentities,
                suites = globallyAvailableInfo.suites
            )
        }
        val ownFullEmpathyCards = activePlayerSlots.filter { slot->
            slot.isKnown(cardsOnTeammatesHands)
        }.flatMap { it.getPossibleIdentities(cardsOnTeammatesHands) }

        val teammates = globallyAvailableInfo.players.filterKeys { it != playerId }.values.map { playerInfo->
            createTeammate(
                playerId = playerInfo.playerId,
                globallyAvailableInfo = globallyAvailableInfo,
                numberOfPlayers = numberOfPlayers,
                activePlayerIndex = activePlayerIndex,
                handMap = handMap,
                teammatePersonalInfo = personalInfo.getInfo(playerInfo.playerId).getOwnInfo(),
                visibleCards = ownFullEmpathyCards + cardsOnTeammatesHandsMap
                    .filterKeys { it != playerInfo.playerId }
                    .values
                    .flatten()
            )
        }.toSet()


        val hand = OwnHand(activePlayerSlots.toSet())

        val pov = ActivePlayerPOV(
            globallyAvailableInfo = globallyAvailableInfo,
            teammates = teammates,
            ownHand = hand,
        )

        return ActivePlayer(
            playerId = playerId,
            playerIndex = activePlayerIndex,
            globallyAvailableInfo = globallyAvailableInfo,
            playerPOV = pov,
            hand = hand,
        )
    }

    fun createTeammate(
        playerId: PlayerId,
        globallyAvailableInfo: GloballyAvailableInfo,
        teammatePersonalInfo: PersonalInfo,
        numberOfPlayers: Int,
        activePlayerIndex: Int,
        handMap: Map<PlayerId, TeammateHand>,
        visibleCards: List<HanabiCard>,
    ): Teammate {
        val playerInfo = globallyAvailableInfo.getPlayerInfo(playerId)
        val teammateVisibleHand = handMap[playerInfo.playerId]
            ?: throw IllegalArgumentException("No hand data on a player with id ${playerInfo.playerId}")
        return Teammate(
            playerId = playerInfo.playerId,
            seatsGap = (numberOfPlayers- activePlayerIndex + playerInfo.playerIndex).mod(numberOfPlayers),
            globallyAvailableInfo = globallyAvailableInfo,
            hand = teammateVisibleHand,
            personalInfo = teammatePersonalInfo,
            visibleCards = visibleCards,
        )
    }
}