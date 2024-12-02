package eelst.ilike.game

import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.utils.Utils

abstract class BaseGloballyAvailableInfo(val playersIds: Collection<PlayerId>)
    : GloballyAvailableInfo {
    override val playingStacks = emptyMap<SuiteId, PlayingStack>()

    override val handsSize =  Utils.getHandSize(playersIds.size)

    override val pace = TODO()

    override val cardsOnStacks: List<HanabiCard>
        get() = TODO("Not yet implemented")
}