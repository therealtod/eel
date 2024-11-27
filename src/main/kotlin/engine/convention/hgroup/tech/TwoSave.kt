package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.action.ObservedAction
import eelst.ilike.engine.action.ObservedClue
import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
import eelst.ilike.engine.player.knowledge.PersonalKnowledge
import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.action.ClueAction
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.*

object TwoSave : SaveClue(
    name = "2-Save",
    appliesTo = setOf(Red, Yellow, Green, Blue, Purple),
) {
    override fun teammateSlotMatchesCondition(teammate: Teammate, slotIndex: Int, playerPOV: PlayerPOV): Boolean {
        val card = teammate.getCardAtSlot(slotIndex)
        return card.rank == Rank.TWO
                && canBeTwoSaved(
            card = card,
            teammates = playerPOV.teammates.filter { it.playerId != teammate.playerId },
        )

    }

    override fun getGameActions(playerPOV: PlayerPOV): Set<ClueAction> {
        val actions = mutableListOf<ClueAction>()
        playerPOV.forEachTeammate { teammate ->
            val chop = getChop(teammate.ownHand)
            if (teammateSlotMatchesCondition(teammate, slotIndex = chop.index, playerPOV)) {
                actions.add(
                    GameActionFactory.createClueAction(
                        clueGiver = playerPOV.playerId,
                        clueReceiver = teammate.playerId,
                        clueValue = Rank.TWO
                    )
                )
            }
        }
        return actions.toSet()
    }

    override fun matchesClue(action: ObservedClue, playerPOV: PlayerPOV): Boolean {
        val clueReceiver = action.gameAction.clueReceiver
        val receiverHand = playerPOV.getHand(clueReceiver)
        val touchedSlotIndexes = action.slotsTouched
        val chop = getChop(receiverHand)
        val focus = getFocusedSlot(
            hand = receiverHand,
            touchedSlotsIndexes = touchedSlotIndexes
        )
        if (focus.index != chop.index) {
            return false
        }
        if (clueReceiver != playerPOV.playerId) {
            val teammate = playerPOV.getTeammate(clueReceiver)
            return teammateSlotMatchesCondition(
                teammate = teammate,
                slotIndex = focus.index,
                playerPOV = playerPOV,
            )
        }
        val saveableTwos = playerPOV
            .globallyAvailableInfo
            .suites
            .flatMap { it.getAllUniqueCards() }
            .filter {
                it.rank == Rank.TWO &&
                playerPOV.globallyAvailableInfo.getGlobalAwayValue(it) > 0 &&
                        canBeTwoSaved(
                            card = it,
                            teammates = playerPOV.teammates
                        )
            }
        val ownSlot = playerPOV.ownHand.getSlot(focus.index)
        return ownSlot.getPossibleIdentities().intersect(saveableTwos).isNotEmpty()
    }

    private fun canBeTwoSaved(
        card: HanabiCard,
        teammates: Collection<Teammate>,
    ): Boolean {
        return teammates.none { teammate ->
                    teammate.hand.copiesOf(card) == 1 &&
                    teammate.getCardAtSlot(getChop(teammate.hand).index) != card
        }
    }

    override fun getGeneratedKnowledge(action: ObservedAction<ClueAction>, playerPOV: PlayerPOV): PersonalKnowledge {
        TODO("Not yet implemented")
    }
}
