package eelst.ilike.engine.strategy

import eelst.ilike.game.GameState


interface GameStateEvaluator {
    fun evaluate(gameState: GameState): Double
}
