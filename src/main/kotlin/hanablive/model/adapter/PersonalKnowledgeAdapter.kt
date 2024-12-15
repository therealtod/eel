package eelst.ilike.hanablive.model.adapter

import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Hand
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

class PersonalKnowledgeAdapter(
    drawActions: Collection<GameDrawActionData>,
    botPlayerIndex: Int,
    playerIndexToPlayerIdMap: Map<Int, PlayerId>,
): PlayerPersonalKnowledge {
    private val botDraws: List<GameDrawActionData>
    private val teammatesDraws: Map<Int, List<GameDrawActionData>>

    init {
        val drawsGroupedByPlayer = drawActions.groupBy { it.playerIndex }
        botDraws = drawsGroupedByPlayer[botPlayerIndex] ?: throw IllegalStateException(
            "No data on cards drawn by the bot could be found"
        )
        teammatesDraws = drawsGroupedByPlayer.minus(botPlayerIndex)
    }

    override fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }

    override fun getOwnHandKnowledge(playerId: PlayerId): PlayersHandKnowledge{
        TODO()
    }

    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        TODO("Not yet implemented")
    }

    override fun canSee(playerId: PlayerId): Boolean {
        TODO("Not yet implemented")
    }
}
