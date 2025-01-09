package eelst.ilike.game.exception

/**
 * Exception indicating that one of the arguments passed to a method refers to a game element not compatible with the
 * game it has been associated to
 */
class ArgumentNotCompatibleWithGameException(message: String): IllegalArgumentException(message)
