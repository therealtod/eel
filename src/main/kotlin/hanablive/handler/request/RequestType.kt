package eelst.ilike.hanablive.handler.request

enum class RequestType(val aliases: List<String>) {
    JOIN(listOf("join")),
    LEAVE(listOf("leave")),
    SET_CONVENTION(listOf("set_convention"));
}
