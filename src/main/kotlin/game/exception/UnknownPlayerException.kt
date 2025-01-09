package game.exception

/**
 * Exception indicating that at attempt to get data about a player could not be satisfied because no played with the
 * required characteristic could be found in the game
 */
class UnknownPlayerException(message: String): NoSuchElementException(message)
