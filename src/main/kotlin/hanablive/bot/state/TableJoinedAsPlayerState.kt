package eelst.ilike.hanablive.bot.state

import eelst.ilike.game.PlayerId
import eelst.ilike.hanablive.bot.HanabLiveBot
import eelst.ilike.hanablive.model.TableId
import eelst.ilike.hanablive.model.dto.command.GameInitData
import eelst.ilike.hanablive.model.dto.command.Table
import eelst.ilike.hanablive.model.dto.instruction.GameActionListData

class TableJoinedAsPlayerState(
    bot: HanabLiveBot,
    commonState: CommonState,
    val tableId: TableId,
) : HanabLiveBotState(
    bot = bot,
    commonState = commonState,
)
