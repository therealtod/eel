use crate::hanablive::UserId;

// Example: welcome {"userID":86718,"username":"ilikeeelst","totalGames":2377,"muted":false,"firstTimeUser":false,"settings":{"desktopNotification":false,"soundMove":0,"soundTimer":true,"keldonMode":false,"colorblindMode":false,"realLifeMode":false,"reverseHands":false,"styleNumbers":false,"showTimerInUntimed":true,"volume":50,"speedrunPreplay":true,"speedrunMode":false,"hyphenatedConventions":true,"createTableVariant":"Brown-Ones \u0026 Orange (6 Suits)","createTableTimed":false,"createTableTimeBaseMinutes":2,"createTableTimePerTurnSeconds":18,"createTableSpeedrun":false,"createTableCardCycle":false,"createTableDeckPlays":false,"createTableEmptyClues":false,"createTableOneExtraCard":false,"createTableOneLessCard":false,"createTableAllOrNothing":false,"createTableDetrimentalCharacters":false,"createTableMaxPlayers":5},"friends":["LI0S"],"playingAtTables":[],"disconSpectatingTable":0,"disconShadowingSeat":0,"randomTableName":"Anselm berate vans","shuttingDown":false,"datetimeShutdownInit":"0001-01-01T00:00:00Z","maintenanceMode":false}
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Welcome {
    #[serde(rename = "userID")]
    pub user_id: UserId,
    pub username: String,
    pub total_games: u32,
    pub muted: bool,
    pub first_time_user: bool,
    pub settings: serde_json::Value,
    pub friends: Vec<String>,
    pub playing_at_tables: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_parses_wire_format() {
        let json = r#"{"userID":86718,"username":"ilikeeelst","totalGames":2377,"muted":false,"firstTimeUser":false,"settings":{},"friends":["LI0S"],"playingAtTables":[],"disconSpectatingTable":0,"disconShadowingSeat":0,"randomTableName":"x","shuttingDown":false,"datetimeShutdownInit":"0001-01-01T00:00:00Z","maintenanceMode":false}"#;
        let w: Welcome = serde_json::from_str(json).expect("welcome should parse");
        assert_eq!(w.user_id, 86718);
        assert_eq!(w.username, "ilikeeelst");
        assert_eq!(w.total_games, 2377);
        assert!(!w.muted);
        assert!(!w.first_time_user);
    }
}
