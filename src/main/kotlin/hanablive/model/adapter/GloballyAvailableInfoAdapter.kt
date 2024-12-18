package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.BaseGloballyAvailableInfo
import eelst.ilike.game.DynamicGloballyAvailableInfo
import eelst.ilike.game.GloballyAvailablePlayerInfo
import eelst.ilike.game.PlayerId
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.TrashPile
import eelst.ilike.game.entity.card.HanabiCard
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.variant.Variant
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.metadata.HanabLiveSuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.HanabLiveVariantMetadata

class GloballyAvailableInfoAdapter(
    gameInitData: GameInitData,
    variant: Variant,
) : BaseGloballyAvailableInfo(
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
