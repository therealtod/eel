package eelst.ilike.hanablive.bot.state

import eelst.ilike.engine.convention.ConventionSet
import eelst.ilike.engine.convention.hgroup.level.Level1
import eelst.ilike.hanablive.model.dto.command.Table

data class CommonState(
    val tables: MutableMap<Int, Table> = emptyMap<Int, Table>().toMutableMap(),
    val conventionSet: ConventionSet = Level1,
)
