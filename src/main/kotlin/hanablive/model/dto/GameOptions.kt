package eelst.ilike.hanablive.model.dto

data class GameOptions(
    val variant: String,
    val emptyClues: Boolean = false,
    val notes: List<List<String>> = emptyList(),
    val characters: List<DetrimentalCharacter> = emptyList(),
)
