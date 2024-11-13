package eelst.ilike.engine.impl

import eelst.ilike.engine.PersonalTeammateInfo
import eelst.ilike.game.VisibleSlot

data class PersonalTeammateInfoImpl(
    val slots: Set<VisibleSlot>
): PersonalTeammateInfo
