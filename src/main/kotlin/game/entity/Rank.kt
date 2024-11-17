package eelst.ilike.game.entity

enum class Rank(val numericalValue: Int) : ClueValue {
    ONE(1),
    TWO(2),
    THREE(3),
    FOUR(4),
    FIVE(5);

    companion object {
        fun getByNumericalValue(numericalValue: Int): Rank {
            require(numericalValue in 1..5) {
                "No Rank with numerical value equal to $numericalValue"
            }
            return entries.first { it.numericalValue == numericalValue }
        }
    }
}
