package eelst.ilike.hanablive

import eelst.ilike.hanablive.entity.TableId
import hanablive.entity.dto.Table
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.ConcurrentMap

data class LobbyState(
    val tables: ConcurrentMap<TableId, Table> = ConcurrentHashMap()
)
