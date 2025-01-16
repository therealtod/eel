package eelst.ilike.engine.knowledge

import eelst.ilike.game.GameUtils
import eelst.ilike.game.GloballyAvailableGameData
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata

object KnowledgeFactory {
    fun createEmptyTeamKnowledge(playersMetadata: List<PlayerMetadata>): TeamKnowledge {
        val handsSize = GameUtils.getHandSize(playersMetadata.size)
        val slotsKnowledge = playersMetadata.map{ _->
            List(handsSize) {
                BaseSlotKnowledge()
            }
        }
        val handsKnowledge = playersMetadata.map{ _ ->
            createEmptyHandKnowledge()
        }
        return createTeamKnowledge(
            slotsKnowledge = slotsKnowledge,
            handsKnowledge = handsKnowledge,
        )
    }

    fun createPlayerKnowledge(
        playerId: PlayerId,
        globallyAvailableGameData: GloballyAvailableGameData,
        cardsVisibleInPlayerHands: Map<PlayerId, Map<Int, HanabiCard>>,
        handKnowledge: Map<PlayerId, HandKnowledge>,
    ): PlayerKnowledge {
        val globallyVisibleCards = globallyAvailableGameData.getCardsOnStacks() +
                globallyAvailableGameData.trashPile.cards
        return PlayerKnowledgeImpl(
            playerId = playerId,
            globallyVisibleCards = globallyVisibleCards,
            cardsVisibleInPlayerHands = cardsVisibleInPlayerHands,
            handKnowledge = handKnowledge
        )
    }

    fun createTeamKnowledge(
        slotsKnowledge: List<List<SlotKnowledge>>,
        handsKnowledge: List<HandKnowledge>
    ): TeamKnowledge {
        return PlayerIndexBasedTeamKnowledge(
            slotsKnowledge = slotsKnowledge,
            handsKnowledge = handsKnowledge,
        )
    }

    fun createEmptyHandKnowledge(): HandKnowledge {
        return HandKnowledgeImplImpl()
    }

    fun createEmptySlotKnowledge(playersMetadata: Collection<PlayerMetadata>): SlotKnowledge {
        TODO()
    }

    fun createSlotKnowledge(visibleCard: HanabiCard, playersMetadata: Collection<PlayerMetadata>): SlotKnowledge {
        TODO()
    }
}
