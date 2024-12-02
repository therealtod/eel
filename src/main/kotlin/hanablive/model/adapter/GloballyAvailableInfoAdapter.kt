package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.GloballyAvailableInfo
import eelst.ilike.game.entity.PlayingStack
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.game.entity.suite.SuiteId
import eelst.ilike.game.variant.Variant
import eelst.ilike.hanablive.model.dto.command.GameActionData
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata

class GloballyAvailableInfoAdapter(
    private val gameInitData: GameInitData,
    private val initialActions: List<GameActionData>,
    variantsMetadata: Collection<VariantMetadata>,
    suitsMetadata: Collection<SuiteMetadata>,
): GloballyAvailableInfo {
    override val clueTokens = TODO()

    override val efficiency = TODO()

    override val handsSize = TODO()

    override val numberOfPlayers = TODO()

    override val pace = TODO()

    override val players = TODO()

    override val playingStacks: Map<SuiteId, PlayingStack>
        get() = TODO("Not yet implemented")

    override val score = 0

    override val strikes = 0

    override val variant = Variant.getVariantByName()
}