package eelst.ilike.game.entity.action


import eelst.ilike.game.entity.ClueValue
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata

open class ClueAction(
    open val clueGiver: PlayerMetadata,
    open val clueReceiver: PlayerMetadata,
    open val value: ClueValue
) : GameAction(actionExecutor = clueGiver, actionType = GameActionType.CLUE)
