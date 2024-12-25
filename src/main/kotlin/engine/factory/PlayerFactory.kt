package eelst.ilike.engine.factory

import eelst.ilike.engine.player.*
import eelst.ilike.engine.player.knowledge.PlayerKnowledge
import eelst.ilike.engine.player.knowledge.TeamKnowledge
import eelst.ilike.game.GameData
import eelst.ilike.game.PlayerMetadata
import eelst.ilike.game.PlayerId
import eelst.ilike.game.SlotMetadata
import eelst.ilike.game.entity.suite.Suit

object PlayerFactory {
    fun createTeammate(
        metadata: PlayerMetadata,
        playerKnowledge: PlayerKnowledge,
        slotData: List<SlotMetadata>,
        suits: Set<Suit>,
    ): Teammate {
        return Teammate(
            playerMetadata = metadata,
            hand = HandFactory.createHand(
                slotData = slotData,
                playerKnowledge = playerKnowledge,
                suits = suits,
            ),
        )
    }

    fun createPlayerPOV(
        playerId: PlayerId,
        gameData: GameData,
        personalKnowledge: TeamKnowledge,
        slotData: Map<PlayerId,List <SlotMetadata>>,
    ): GameFromPlayerPOV {
        return BasePlayerPOV(
            playerId = playerId,
            gameData = gameData,
            teamKnowledge = personalKnowledge,
            slotData = slotData,
            )
    }
}
