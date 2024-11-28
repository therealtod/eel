package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.GameAction


interface ConventionTech {
    val name: String
    fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean
    fun getGameActions(playerPOV: PlayerPOV): Set<GameAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun matches(action: ObservedAction, playerPOV: PlayerPOV): Boolean
    fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): PersonalKnowledge
}
