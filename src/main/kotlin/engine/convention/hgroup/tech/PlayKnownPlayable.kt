package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.PlayTech
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.GameState
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.variant.Variant

object PlayKnownPlayable : HGroupTech("Play Known Playable"), PlayTech {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun getUpdatedKnowledge(playAction: PlayAction, currentKnowledge: TeamKnowledge): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getGameActions(gameState: GameState, currentKnowledge: TeamKnowledge): Set<GameAction> {
        TODO()
    }

    override fun matchesPlay(playAction: PlayAction, gameState: GameState, currentKnowledge: TeamKnowledge): Boolean {
        TODO("Not yet implemented")
    }
}
