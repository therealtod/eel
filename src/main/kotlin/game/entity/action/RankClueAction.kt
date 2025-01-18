package eelst.ilike.game.entity.action


import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata

data class RankClueAction(
    override val clueGiver: PlayerMetadata,
    override val clueReceiver: PlayerMetadata,
    val rank: Rank
) : ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = rank
)
