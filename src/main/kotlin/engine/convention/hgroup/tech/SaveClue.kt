package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.engine.hand.InterpretedHand


sealed class SaveClue(name: String) : HGroupClue(name) {
    override fun overrides(otherTech: ConventionTech): Boolean {
        return otherTech !is SaveClue
    }

    override fun matchesClueBySlot(focusIndex: Int, hand: InterpretedHand): Boolean {
        val chop = getChop(hand)
        return focusIndex == chop.index
    }
}
