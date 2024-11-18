package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.ConventionTech
import eelst.ilike.engine.convention.ConventionalAction
import eelst.ilike.engine.convention.hgroup.HGroupCommon
import eelst.ilike.engine.player.Teammate
import eelst.ilike.game.action.Clue
import eelst.ilike.game.action.ColorClue
import eelst.ilike.game.action.RankClue
import eelst.ilike.game.entity.Color
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.Slot
import eelst.ilike.game.entity.card.HanabiCard

abstract class HGroupTech(
    override val name: String,
    private val takesPrecedenceOver: Set<HGroupTech>,
) : ConventionTech {
    protected fun getAllFocusingClues(
        card: HanabiCard,
        slot: Slot,
        teammate: Teammate,
    ): Set<Clue> {
        val ranks = card.getRanksTouchingCard()
        val colors = card.getColorsTouchingCard()
        return getRankCluesFocusing(
            slot = slot,
            teammate = teammate,
            ranks = ranks,
        ) +
                getColorCluesFocusing(
                    slot = slot,
                    teammate = teammate,
                    colors = colors,
                )
    }

    protected fun getAllFocusingActions(
        card: HanabiCard,
        slot: Slot,
        teammate: Teammate,
    ): Set<ConventionalAction> {
        val clues = getAllFocusingClues(
            card = card,
            slot = slot,
            teammate = teammate
        )
        return clues.map {
            ConventionalAction(
                action = it,
                tech = this
            )
        }.toSet()
    }

    private fun getRankCluesFocusing(
        slot: Slot,
        teammate: Teammate,
        ranks: Set<Rank>,
    ): Set<Clue> {
        return ranks.map {
            RankClue(rank = it, receiver = teammate.playerId)
        }
            .filter { HGroupCommon.getClueFocusSlotIndex(clue = it, hand = teammate.hand) == slot.index }
            .toSet()
    }

    private fun getColorCluesFocusing(
        slot: Slot,
        teammate: Teammate,
        colors: Set<Color>,
    ): Set<Clue> {
        return colors.map {
            ColorClue(color = it, receiver = teammate.playerId)
        }
            .filter { HGroupCommon.getClueFocusSlotIndex(clue = it, hand = teammate.hand) == slot.index }
            .toSet()
    }

    override fun overrides(otherTech: ConventionTech): Boolean {
        return takesPrecedenceOver.contains(otherTech)
    }

    override fun toString() = name
}

