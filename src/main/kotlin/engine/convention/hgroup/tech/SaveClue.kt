package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.player.GameFromPlayerPOV
import eelst.ilike.game.entity.Hand


sealed class SaveClue(name: String) : HGroupClue(name) {
    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue
    }

    override fun matchesClueBySlot(focusIndex: Int, hand: Hand, playerPOV: GameFromPlayerPOV): Boolean {
        val chop = getChop(hand, playerPOV)
        return focusIndex == chop.index
    }
}
