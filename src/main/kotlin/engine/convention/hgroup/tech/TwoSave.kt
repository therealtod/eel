package eelst.ilike.engine.convention.hgroup.tech

import eelst.ilike.engine.convention.hgroup.HGroupCommon.getChop
import eelst.ilike.engine.factory.GameActionFactory
import eelst.ilike.engine.player.PlayerPOV
import eelst.ilike.engine.player.Teammate
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
            teammate = teammate,
            playerPOV = playerPOV,
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

    private fun canBeTwoSaved(
        card: HanabiCard,
        teammate: Teammate,
        playerPOV: PlayerPOV,
    ): Boolean {
        return playerPOV.teammates.none { otherTeammate ->
            otherTeammate.playerId != teammate.playerId &&
                    otherTeammate.hand.copiesOf(card) == 1 &&
                    otherTeammate.getCardAtSlot(getChop(otherTeammate.hand).index) != card
        }
    }
}
