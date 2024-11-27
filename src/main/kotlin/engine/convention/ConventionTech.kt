package eelst.ilike.engine.convention

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.action.ObservedDiscard
import eelst.ilike.engine.action.ObservedPlay
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.action.GameAction


interface ConventionTech<T : GameAction> {
    val name: String
    fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean
    fun getGameActions(playerPOV: PlayerPOV): Set<T>
    fun overrides(otherTech: ConventionTech<T>): Boolean
    fun matches(action: ObservedAction<T>, playerPOV: PlayerPOV): Boolean
    fun matchesPlay(action: ObservedPlay, playerPOV: PlayerPOV): Boolean
    fun matchesDiscard(action: ObservedDiscard, playerPOV: PlayerPOV): Boolean
    fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean
    fun getGeneratedKnowledge(action: ObservedAction<T>, playerPOV: PlayerPOV): PersonalKnowledge
}
