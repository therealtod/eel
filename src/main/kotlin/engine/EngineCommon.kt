package eelst.ilike.engine

import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.hand.InterpretedHand
import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.action.GameAction

object EngineCommon {
    fun <T: GameAction> getPrunedAction(actions: Collection<ConventionalAction<T>>): Set<ConventionalAction<T>> {
        val groupedActions = actions.groupBy { it.action }
        return groupedActions.map {
            it.value.fold(it.value.first()) { curr, next ->
                if (curr.tech.overrides(next.tech)) curr else next
            }
        }.toSet()
    }

    fun getTouchedSlots(clueValue: ClueValue, hand: InterpretedHand): Set<Int> {
        return hand.getSlotsTouchedBy(clueValue).map { it.index }.toSet()
    }
}
