package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.VisibleTeammate
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.engine.player.knowledge.PlayerPersonalKnowledge
import eelst.ilike.game.entity.action.GameAction


interface ConventionTech {
    val name: String
    fun teammateSlotMatchesCondition(teammate: VisibleTeammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean
    fun getGameActions(playerPOV: PlayerPOV): Set<GameAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun matches(action: ObservedAction, playerPOV: PlayerPOV): Boolean
    fun getGeneratedKnowledge(action: ObservedAction, playerPOV: PlayerPOV): Knowledge
}
