package eelst.ilike.engine.player.knowledge

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.game.PlayerId

class PlayersHandKnowledge: PlayerPersonalKnowledge {
    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        TODO("Not yet implemented")
    }

    override fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge {
        return this
    }

    override fun canSee(playerId: PlayerId): Boolean {
        TODO("Not yet implemented")
    }

    override fun getVisibleHand(playerId: PlayerId): VisibleHand {
        TODO("Not yet implemented")
    }

    override fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge {
        TODO("Not yet implemented")
    }
}