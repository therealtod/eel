package eelst.ilike.hanablive.bot.dto

import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
data class HanabLiveBotConfiguration (
    val userCommandPrefix: String,
)
