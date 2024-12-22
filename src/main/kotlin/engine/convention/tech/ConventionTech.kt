package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.PlayAction


interface ConventionTech {
    val name: String
    fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: GameFromPlayerPOV): Boolean
    fun getGameActions(playerPOV: GameFromPlayerPOV): Set<GameAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun matches(playAction: PlayAction, playerPOV: GameFromPlayerPOV): Boolean
    fun matches(discardAction: DiscardAction, playerPOV: GameFromPlayerPOV): Boolean
    fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: GameFromPlayerPOV): Boolean
    fun getGeneratedKnowledge(playAction: PlayAction, playerPOV: GameFromPlayerPOV): Knowledge
    fun getGeneratedKnowledge(discardAction: DiscardAction, playerPOV: GameFromPlayerPOV): Knowledge
    fun getGeneratedKnowledge(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: GameFromPlayerPOV): Knowledge
}
