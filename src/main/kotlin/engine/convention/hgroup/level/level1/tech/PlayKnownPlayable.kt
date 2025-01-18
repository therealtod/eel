package eelst.ilike.engine.convention.hgroup.level.level1.tech

import eelst.ilike.engine.convention.hgroup.tech.HGroupTech
import eelst.ilike.engine.convention.tech.PlayTech
import eelst.ilike.engine.knowledge.TeamKnowledge
import eelst.ilike.game.gamestate.GameState
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.action.PlayAction
import eelst.ilike.game.entity.player.PlayerMetadata
import eelst.ilike.game.entity.variant.Variant

object PlayKnownPlayable : HGroupTech("Play Known Playable"), PlayTech {
    override fun appliesTo(card: HanabiCard, variant: Variant): Boolean {
        return true
    }

    override fun getUpdatedKnowledge(playAction: PlayAction, currentKnowledge: TeamKnowledge): TeamKnowledge {
        return currentKnowledge
    }

    override fun getGameActions(gameState: GameState, currentKnowledge: TeamKnowledge): Collection<PlayAction> {
        val actions = mutableListOf<PlayAction>()
        val playerOnTurn = gameState.globallyAvailableGameData.getPlayerOnTurn()
        playerOnTurn.forEachSlot { slotIndex, _ ->
            if (slotMatchesCondition(
                slotIndex = slotIndex,
                playerMetadata = playerOnTurn.getMetadata(),
                gameState =gameState,
                currentKnowledge = currentKnowledge,
            )) {
                val action = PlayAction(
                    playerMetadata = playerOnTurn.getMetadata(),
                    slotIndex = slotIndex,
                )
                actions.add(action)
            }
        }
        return actions
    }

    override fun matchesPlay(playAction: PlayAction, gameState: GameState, currentKnowledge: TeamKnowledge): Boolean {
        return slotMatchesCondition(
            slotIndex = playAction.slotIndex,
            playerMetadata = playAction.playerMetadata,
            gameState = gameState,
            currentKnowledge = currentKnowledge,
        )
    }

    override fun slotMatchesCondition(
        slotIndex: Int,
        playerMetadata: PlayerMetadata,
        gameState: GameState,
        currentKnowledge: TeamKnowledge
    ): Boolean {
        val slotKnowledge = currentKnowledge.getSlotKnowledge(playerMetadata.playerIndex, slotIndex)
        return slotKnowledge.slotIsKnownByOwner()
                && gameState.globallyAvailableGameData.isImmediatelyPlayable(slotKnowledge.getInferredIdentity())
    }
}
