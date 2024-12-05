package eelst.ilike.hanablive.model.adapter

import eelst.ilike.engine.factory.KnowledgeFactory
import eelst.ilike.engine.hand.VisibleHand
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledge
import eelst.ilike.engine.player.knowledge.PersonalHandKnowledgeImpl
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.hanablive.model.dto.instruction.GameDrawActionData

class PersonalKnowledgeAdapter(
    drawActions: Collection<GameDrawActionData>,
    botPlayerIndex: Int,
    playerIndexToPlayerIdMap: Map<Int, PlayerId>,
): PersonalKnowledge {
    private val botDraws: List<GameDrawActionData>
    private val teammatesDraws: Map<Int, List<GameDrawActionData>>
    private val visibleHands: Map<PlayerId, VisibleHand>

    init {
        val drawsGroupedByPlayer = drawActions.groupBy { it.playerIndex }
        botDraws = drawsGroupedByPlayer[botPlayerIndex] ?: throw IllegalStateException(
            "No data on cards drawn by the bot could be found"
        )
        teammatesDraws = drawsGroupedByPlayer.minus(botPlayerIndex)
        visibleHands = teammatesDraws.map {
            Pair(
                playerIndexToPlayerIdMap[it.key] ?: throw IllegalStateException("No player with player index ${it.key}"),
                VisibleHand(
                    TODO()
                )
            )
        }.toMap()
    }

    override fun accessibleTo(playerId: PlayerId): PersonalKnowledge {
        TODO("Not yet implemented")
    }

    override fun getOwnHandKnowledge(playerId: PlayerId): PersonalHandKnowledge {
        return PersonalHandKnowledgeImpl(emptyMap())
    }

    override fun getVisibleHand(playerId: PlayerId): VisibleHand {
        return visibleHands[playerId]
            ?: throw IllegalArgumentException("No player $playerId with visible hand could be found")
    }

    override fun getSlotIdentity(slotIndex: Int, playerId: PlayerId): HanabiCard {
        TODO("Not yet implemented")
    }
}