package eelst.ilike.game.entity.suite

import eelst.ilike.game.entity.Color

class ClassicSuit(
    id: String,
    name: String,
    abbreviations: List<String>,
    private val definingColor: Color,
)
    : BaseClassicSuite(
    id = id,
        name = name,
        abbreviations = abbreviations,
) {

    override fun getAssociatedColors(): Collection<Color> {
        return listOf(definingColor)
    }
}
