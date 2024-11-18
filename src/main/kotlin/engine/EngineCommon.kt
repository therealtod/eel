package eelst.ilike.engine

import eelst.ilike.engine.convention.ConventionalAction

object EngineCommon {
    fun getPrunedAction(actions: Collection<ConventionalAction>): Set<ConventionalAction> {
        val groupedActions = actions.groupBy { it.action }
        return groupedActions.map {
            it.value.fold(it.value.first()) { curr, next ->
                if (curr.tech.overrides(next.tech)) curr else next
            }
        }.toSet()
    }
}