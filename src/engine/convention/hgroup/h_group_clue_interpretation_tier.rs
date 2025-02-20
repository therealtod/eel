/// Describes the priority that a certain convention tech takes when interpreting a teammate's 
/// action
enum HGroupActionInterpretationTier {
    Default, // Save clue
    SimplePlayClue, // Direct play clue, Delayed play clue, Prompt on teammate
    Finesse,
}