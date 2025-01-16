package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.tech.ConventionTech
import eelst.ilike.game.entity.HanabiCard
import eelst.ilike.game.entity.variant.Variant

abstract class HGroupTech(override val name: String): ConventionTech {
    abstract fun appliesTo(card: HanabiCard, variant: Variant): Boolean
}