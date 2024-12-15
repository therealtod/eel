package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.game.PlayerId

class PlayerPersonalKnowledgeImpl(
    private val personalHandKnowledge: Map<PlayerId, PersonalHandKnowledge>,
    private val visibleHands: Map<PlayerId, VisibleHand> = emptyMap()
) : PlayerPersonalKnowledge {
    private val handSize = visibleHands.values.first().size

    override fun canSee(playerId: PlayerId): Boolean {
        return visibleHands.keys.contains(playerId)
    }

    override fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge {
        return personalHandKnowledge[playerId]!!
    }

    override fun getVisibleHand(playerId: PlayerId): VisibleHand {
        return visibleHands[playerId]!!
    }

    override fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge {
        return PlayerPersonalKnowledgeImpl(
            personalHandKnowledge = personalHandKnowledge,
            visibleHands = visibleHands.minus(playerId)
        )
    }

    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        require(knowledge is PlayerPersonalKnowledge) {
            "Illegal knowledge update: Types not compatible"
        }
        TODO()
    }
}
