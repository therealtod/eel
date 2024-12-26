package eelst.ilike.game.entity.clue

enum class ColorClueValue : ClueValue {
    RED,
    YELLOW,
    GREEN,
    BLUE,
    PURPLE,
    TEAL,
    BLACK,
    BROWN,
    PINK;

    companion object {
        fun getFromStringFormat(color: String): ColorClueValue {
            return entries.find { it.name.equals(color, ignoreCase = true) }
                ?: throw IllegalArgumentException(
                    "Could not find a color corresponding to the string $color"
                )
        }
    }
}