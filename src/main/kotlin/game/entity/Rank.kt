package eelst.ilike.game.entity

enum class Rank(val numericalValue: Int) : CardAttribute {
    ONE(1),
    TWO(2),
    THREE(3),
    FOUR(4),
    FIVE(5);

    companion object {
        fun getByNumericalValue(numericalValue: Int): Rank {
            require(numericalValue > 0 && numericalValue <= 5) {
                "No Rank with numerical value equal to $numericalValue"
            }
            return entries.first { it.numericalValue == numericalValue }
        }
    }
}