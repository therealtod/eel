package eelst.ilike.hanablive.entity.dto.instruction

import eelst.ilike.hanablive.entity.TableId


/**
 * @param tableID the id of the hanab.live table where the game is played
 * @param list contains all the actions (in the hanab.live format) that have been already performed up to the moment the
 * game is joined
 */
data class GameActionListData(
    val tableID: TableId,
    val list: List<HanabLiveGameActionData>
)

