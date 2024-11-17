package eelst.ilike.game.entity.action

import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.Rank

data class RankClueAction(
    override val clueGiver: PlayerId,
    override val clueReceiver: PlayerId,
    val rank: Rank
) : ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = rank
)
