package eelst.ilike.engine.action

import eelst.ilike.game.entity.clue.Clue

class ObservedClue(executedAction: ExecutedAction<Clue>): ObservedAction<Clue>(executedAction) {
    override fun getAction(): Clue {
        return executedAction.getAction()
    }
}