package eelst.ilike.hanablive.model.adapter

import eelst.ilike.game.entity.Rank
import eelst.ilike.game.entity.suite.Suite
import eelst.ilike.hanablive.model.dto.metadata.SuiteMetadata
import eelst.ilike.hanablive.model.dto.metadata.VariantMetadata

class SuiteAdapter(suiteMetadata: SuiteMetadata, variantMetadata: VariantMetadata)
    : Suite(
        id = suiteMetadata.id,
        name = suiteMetadata.name,
        abbreviations = listOf(suiteMetadata.abbreviation),
        specialRanks = if(variantMetadata.upOrDown) setOf(Rank.START) else emptySet(),
        stackSize = variantMetadata.stackSize
    )
