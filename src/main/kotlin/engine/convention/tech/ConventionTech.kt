package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.action.DiscardAction
import eelst.ilike.game.entity.action.GameAction
import eelst.ilike.game.entity.action.PlayAction


interface ConventionTech {
    val name: String
    fun teammateSlotMatchesCondition(teammate: Teammate, slot: Slot, playerPOV: PlayerPOV): Boolean
    fun getGameActions(playerPOV: PlayerPOV): Set<GameAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun matches(playAction: PlayAction, playerPOV: PlayerPOV): Boolean
    fun matches(discardAction: DiscardAction, playerPOV: PlayerPOV): Boolean
    fun matches(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: PlayerPOV): Boolean
    fun getGeneratedKnowledge(playAction: PlayAction, playerPOV: PlayerPOV): Knowledge
    fun getGeneratedKnowledge(discardAction: DiscardAction, playerPOV: PlayerPOV): Knowledge
    fun getGeneratedKnowledge(clueAction: ClueAction, touchedSlotsIndexes: Set<Int>, playerPOV: PlayerPOV): Knowledge
}
