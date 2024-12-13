package eelst.ilike.engine.player

/*
class ActivePlayer(
    playerId: PlayerId,
    playerIndex: Int,
    globallyAvailableInfo: GloballyAvailableInfoImpl,
    personalKnowledge: PersonalKnowledge,
) : ConventionsUsingPlayer(
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
        PlayerFactory.createVisibleTeammate(
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

    fun getLegalActions(conventionSet: BaseConventionSet): Set<ConventionalAction> {
        return getCandidateActions(conventionSet.getTechs()).toSet()
    }

    override fun hasCardInSlot(card: HanabiCard, slotIndex: Int): Boolean {
        return getOwnSlot(slotIndex).contains(card)
    }

    private fun getCandidateActions(
        techs: Collection<ConventionTech>
    ): Collection<ConventionalAction> {
        return techs
            .flatMap { tech ->
                tech.getGameActions(playerPOV,)
                    .map {
                        ConventionalAction(
                            action = it,
                            tech = tech,
                        )
                    }
            }
    }

    private fun prune(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        val overlappingGroups = actions.groupBy { it.action }
        return overlappingGroups.map { group ->
            group.value.fold(listOf(group.value.first())) { curr, next ->
                if (curr.any { it.tech.overrides(next.tech) })
                    curr
                else
                    curr + next
            }
        }.flatten()
            .toSet()
    }
}

 */
