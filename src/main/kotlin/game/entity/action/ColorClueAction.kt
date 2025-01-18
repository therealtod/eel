package eelst.ilike.game.entity.action


import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.player.PlayerId
import eelst.ilike.game.entity.player.PlayerMetadata

data class ColorClueAction(
    override val clueGiver: PlayerMetadata,
    override val clueReceiver: PlayerMetadata,
    val color: Color
) : ClueAction(
    clueGiver = clueGiver,
    clueReceiver = clueReceiver,
    value = color
)
