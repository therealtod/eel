package eelst.ilike.engine.strategy

import eelst.ilike.game.Game

interface GameStateEvaluator {
    fun evaluate(gameState: Game): Double
}