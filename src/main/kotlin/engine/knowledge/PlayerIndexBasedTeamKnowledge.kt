package eelst.ilike.engine.knowledge

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.DrawAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerId

class PlayerIndexBasedTeamKnowledge(
    private val slotsKnowledge: List<List<SlotKnowledge>>,
    private val handsKnowledge: List<HandKnowledge>,
) : TeamKnowledge {
    override fun getSlotKnowledge(playerId: PlayerId, slotIndex: Int) {
        TODO("Not yet implemented")
    }

    override fun getAfter(drawAction: DrawAction): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfter(drawAction: DrawAction, card: HanabiCard): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfter(playAction: PlayAction, conventionSet: ConventionSet): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfter(playAction: PlayAction, playedCard: HanabiCard, conventionSet: ConventionSet): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfter(discardAction: DiscardAction, conventionSet: ConventionSet): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfter(
        discardAction: DiscardAction,
        discardedCard: HanabiCard,
        conventionSet: ConventionSet
    ): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getAfter(
        clueAction: ClueAction,
        touchedSlotsIndexes: Collection<Int>,
        conventionSet: ConventionSet
    ): TeamKnowledge {
        TODO("Not yet implemented")
    }
}
