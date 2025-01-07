package eelst.ilike.game.entity


enum class Color : ClueValue {
    RED,
    YELLOW,
    GREEN,
    BLUE,
    PURPLE,
    TEAL;

    companion object {
        fun getFromStringFormat(color: String): Color {
            return entries.find { it.name.equals(color, ignoreCase = true) }
                ?: throw IllegalArgumentException(
                    "Could not find a color corresponding to the string $color"
                )
        }
    }
}
