package eelst.ilike.hanablive.model.adapter

import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

class PersonalKnowledgeAdapter(
    drawActions: Collection<GameDrawActionData>,
    botPlayerIndex: Int,
    playerIndexToPlayerIdMap: Map<Int, PlayerId>,
): TeamKnowledge {
    private val botDraws: List<GameDrawActionData>
    private val teammatesDraws: Map<Int, List<GameDrawActionData>>

    init {
        val drawsGroupedByPlayer = drawActions.groupBy { it.playerIndex }
        botDraws = drawsGroupedByPlayer[botPlayerIndex] ?: throw IllegalStateException(
            "No data on cards drawn by the bot could be found"
        )
        teammatesDraws = drawsGroupedByPlayer.minus(botPlayerIndex)
    }

    override fun getAsSeenBy(playerId: PlayerId): TeamKnowledge {
        TODO("Not yet implemented")
    }

    override fun getPlayerKnowledge(playerId: PlayerId): PlayerKnowledge {
        TODO("Not yet implemented")
    }
}
