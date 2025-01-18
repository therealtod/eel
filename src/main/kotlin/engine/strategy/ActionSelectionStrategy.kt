package eelst.ilike.engine.strategy

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.game.gamestate.GameState

/**
 * Defines the strategy on how to select the [ConventionalAction] the player on turn would take
 */
interface ActionSelectionStrategy {
    /**
     * @return the [ConventionalAction] the player on turn should take given the [gameState] and according to the
     * [conventionSet] that the team has agreed on
     */
    fun selectAction(gameState: GameState, conventionSet: ConventionSet): ConventionalAction
}
