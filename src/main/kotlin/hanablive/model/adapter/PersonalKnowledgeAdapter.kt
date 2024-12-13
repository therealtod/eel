package eelst.ilike.hanablive.model.adapter

import eelst.ilike.engine.hand.slot.VisibleHand
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledge
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
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

    val visibleHands: Map<PlayerId, VisibleHand>
    init {
        val drawsGroupedByPlayer = drawActions.groupBy { it.playerIndex }
        botDraws = drawsGroupedByPlayer[botPlayerIndex] ?: throw IllegalStateException(
            "No data on cards drawn by the bot could be found"
        )
        teammatesDraws = drawsGroupedByPlayer.minus(botPlayerIndex)
        visibleHands = teammatesDraws.map {
                VisibleHand(
                    ownerId = playerIndexToPlayerIdMap[it.key] ?: throw IllegalStateException("No player with player index ${it.key}"),
                    slots = TODO()
                )
        }.associateBy { it.ownerId }
    }

    override fun canSee(playerId: PlayerId): Boolean {
        return visibleHands.keys.contains(playerId)
    }

    override fun accessibleTo(playerId: PlayerId): PlayerPersonalKnowledge {
        TODO("Not yet implemented")
    }

    override fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge {
        return PersonalHandKnowledgeImpl(botDraws.size, emptyMap())
    }

    override fun getVisibleHand(playerId: PlayerId): VisibleHand {
        TODO("Not yet implemented")
    }

    override fun getUpdatedWith(knowledge: Knowledge): Knowledge {
        TODO("Not yet implemented")
    }
}
