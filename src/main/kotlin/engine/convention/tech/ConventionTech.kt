package eelst.ilike.engine.convention.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.player.ActivePlayer
import eelst.ilike.engine.player.EngineHandlerPlayer
import eelst.ilike.engine.player.knowledge.Knowledge
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.action.GameAction


interface ConventionTech {
    val name: String
    fun teammateSlotMatchesCondition(engineHandlerPlayer: EngineHandlerPlayer, slot: Slot, activePlayer: ActivePlayer): Boolean
    fun getGameActions(activePlayer: ActivePlayer): Set<GameAction>
    fun overrides(otherTech: ConventionTech): Boolean
    fun matches(action: ObservedAction, activePlayer: ActivePlayer): Boolean
    fun getGeneratedKnowledge(action: ObservedAction, activePlayer: ActivePlayer): Knowledge
}
