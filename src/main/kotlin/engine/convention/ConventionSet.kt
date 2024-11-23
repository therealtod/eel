package eelst.ilike.engine.convention

abstract class ConventionSet(
    val name: String,
    val includes: Set<ConventionSet> = emptySet(),
    private val techs: Set<ConventionTech<*>>
) {
    fun getTechs(): Set<ConventionTech<*>> {
        return techs + includes.flatMap { it.getTechs() }
    }
}
