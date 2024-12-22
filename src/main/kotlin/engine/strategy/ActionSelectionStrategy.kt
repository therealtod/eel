package eelst.ilike.engine.strategy

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.player.GameFromPlayerPOV

interface ActionSelectionStrategy {
    fun selectAction(playerPOV: GameFromPlayerPOV, conventionSet: ConventionSet): ConventionalAction
}
