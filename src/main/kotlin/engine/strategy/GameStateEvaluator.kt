package eelst.ilike.engine.strategy

import eelst.ilike.game.gamestate.GameState


interface GameStateEvaluator {
    fun evaluate(gameState: GameState): Double
}
