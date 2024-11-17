package eelst.ilike.engine.impl

import eelst.ilike.engine.PersonalTeammateInfo
import eelst.ilike.game.VisibleSlot

data class PersonalTeammateInfoImpl(
    override val slots: Set<VisibleSlot>
): PersonalTeammateInfo
