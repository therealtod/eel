package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.game.PlayerId

class PersonalKnowledgeImpl(
    private val personalHandKnowledge: Map<PlayerId, PersonalHandKnowledge>,
    private val visibleHands: Map<PlayerId, VisibleHand> = emptyMap()
) : PersonalKnowledge {
    override fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge {
        return personalHandKnowledge[playerId]!!
    }

    override fun getVisibleHand(playerId: PlayerId): VisibleHand {
        return visibleHands[playerId]!!
    }

    override fun getVisibleHands(): Map<PlayerId, VisibleHand> {
        return visibleHands
    }

    override fun accessibleTo(playerId: PlayerId): PersonalKnowledge {
        return PersonalKnowledgeImpl(
            personalHandKnowledge = personalHandKnowledge,
            visibleHands = visibleHands.minus(playerId)
        )
    }
}
