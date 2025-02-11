pub trait ConventionTech {
    fn get_game_actions();
    fn matches_play() -> bool;
    fn matches_discard() -> bool;
    fn matches_clue() -> bool;
    fn get_acquired_knowledge();
    fn slot_matches_tech_defining_condition() -> bool;
}
