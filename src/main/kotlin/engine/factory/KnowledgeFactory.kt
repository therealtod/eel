package eelst.ilike.engine.factory

import eelst.ilike.engine.convention.hgroup.signal.Signal
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.knowledge.*
import eelst.ilike.game.entity.card.HanabiCard

object KnowledgeFactory {
    fun createEmptyTeamKnowledge(playerPOV: GameFromPlayerPOV): TeamKnowledge {
        val gameData = playerPOV.getGameData()
        return TeamKnowledgeFromPlayerPOV(
            povPlayerId = playerPOV.getOwnPlayerId(),
            globallyVisibleCards = gameData.getCardsOnStacks() + gameData.trashPile.cards,
            playersHandsKnowledge = playerPOV.getPlayers().mapValues { HandKnowledgeImpl() }
        )
    }

    fun createEmptyHandKnowledge(): HandKnowledge {
        return HandKnowledgeImpl()
    }

    fun createSlotKnowledge(
        visibleCard: HanabiCard? = null,
        signals: Map<Int, Signal> = emptyMap(),
        impliedIdentities: Set<HanabiCard> = emptySet(),
        hasConflictingInformation: Boolean = false,
    ): SlotKnowledge {
        return if (visibleCard == null) {
            BaseSlotKnowledge(
                signals = signals.toMutableMap(),
                impliedIdentities = impliedIdentities,
                hasConflictingInformation = hasConflictingInformation
            )
        } else {
            VisibleSlotKnowledge(
                visibleCard = visibleCard,
                signals = signals.toMutableMap(),
                impliedIdentities = impliedIdentities,
                hasConflictingInformation = hasConflictingInformation
            )
        }
    }
}
