package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.BaseGame
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.variant.Variant
import eelst.ilike.hanablive.model.dto.command.GameInitData

class GameAdapter(
    gameInitData: GameInitData,
    variant: Variant,
) : BaseGame(
    variant = variant,
    playingStacks = variant.suits.associate { it.id to PlayingStack(emptyList(), it) },
    trashPile = TrashPile(emptyList()),
    strikes = 0,
    clueTokens = 8,
    players = gameInitData.playerNames.mapIndexed { index, p -> GloballyAvailablePlayerInfo(
        playerId = p,
        playerIndex = index
    ) }.associateBy { it.playerId }
)
