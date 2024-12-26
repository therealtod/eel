package eelst.ilike.hanablive.bot.state

import eelst.ilike.hanablive.model.dto.command.Table

data class HanabLiveLobbyState(
    val tables: MutableMap<Int, Table> = emptyMap<Int, Table>().toMutableMap(),
)
