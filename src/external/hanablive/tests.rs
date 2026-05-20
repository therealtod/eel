#[cfg(test)]
mod dto_tests {
    use crate::external::hanablive::dto::chat_message::ChatMessage;
    use crate::external::hanablive::dto::game_init_data::GameInitData;
    use crate::external::hanablive::dto::game_options::GameOptions;
    use crate::external::hanablive::dto::instruction::game_action_data::{
        ClueActionData, DiscardActionData, DrawActionData, GameActionData, PlayActionData,
        StatusActionData, StrikeActionData, TurnActionData,
    };
    use crate::external::hanablive::dto::instruction::game_action_list::GameAction;
    use crate::external::hanablive::dto::instruction::game_action_list::GameActionListData;
    use crate::external::hanablive::dto::instruction::game_action_type::GameActionType;
    use crate::external::hanablive::dto::instruction_type::InstructionType;
    use crate::external::hanablive::dto::metadata::suit_metadata::SuitMetadata;
    use crate::external::hanablive::dto::metadata::variant_metadata::VariantMetadata;
    use crate::external::hanablive::dto::outgoing::{
        GetGameInfo1, GetGameInfo2, Instruction, PasswordProtectedTableJoin, TableJoin,
    };
    use crate::external::hanablive::dto::table::Table;

    const GAME_INIT_PAYLOAD: &str = include_str!("test_fixtures/game_init_payload.json");
    const GAME_ACTION_LIST_PAYLOAD: &str =
        include_str!("test_fixtures/game_action_list_payload.json");
    const TABLE_LIST_PAYLOAD: &str = include_str!("test_fixtures/table_list.json");
    const TABLE_PAYLOAD: &str = include_str!("test_fixtures/table.json");
    const CHAT_MESSAGE_PAYLOAD: &str = include_str!("test_fixtures/chat_message.json");
    const GAME_ACTION_PLAY: &str = include_str!("test_fixtures/game_action_play.json");
    const GAME_ACTION_CLUE: &str = include_str!("test_fixtures/game_action_clue.json");
    const GAME_ACTION_DISCARD: &str = include_str!("test_fixtures/game_action_discard.json");
    const VARIANTS_METADATA: &str = include_str!("test_fixtures/variants_metadata.json");
    const SUITS_METADATA: &str = include_str!("test_fixtures/suits_metadata.json");

    #[test]
    fn test_instruction_type_from_string() {
        assert_eq!(
            InstructionType::from_string("chat"),
            Some(InstructionType::Chat)
        );
        assert_eq!(
            InstructionType::from_string("tableList"),
            Some(InstructionType::TableList)
        );
        assert_eq!(
            InstructionType::from_string("gameAction"),
            Some(InstructionType::GameAction)
        );
        assert_eq!(
            InstructionType::from_string("gameActionList"),
            Some(InstructionType::GameActionList)
        );
        assert_eq!(
            InstructionType::from_string("init"),
            Some(InstructionType::Init)
        );
        assert_eq!(
            InstructionType::from_string("tableStart"),
            Some(InstructionType::TableStart)
        );
        assert_eq!(
            InstructionType::from_string("welcome"),
            Some(InstructionType::Welcome)
        );
        assert_eq!(InstructionType::from_string("unknown"), None);
    }

    #[test]
    fn test_deserialize_game_init_data() {
        let data: GameInitData = serde_json::from_str(GAME_INIT_PAYLOAD).unwrap();
        assert_eq!(data.table_id, 37956);
        assert_eq!(data.player_names, vec!["Alice", "Bob", "Cathy"]);
        assert_eq!(data.our_player_index, 0);
        assert!(data.spectating);
        assert!(!data.shadowing);
        assert_eq!(data.seed, "p3v2s14");
        assert_eq!(data.options.variant_name, "6 Suits");
        assert_eq!(data.options.num_players, 3);
        assert_eq!(data.shared_replay_leader, "awwest2");
        assert_eq!(data.shared_replay_segment, 35);
    }

    #[test]
    fn test_deserialize_game_options() {
        let options: GameOptions = serde_json::from_str(
            r#"{
            "numPlayers": 4,
            "startingPlayer": 1,
            "variantID": 10,
            "variantName": "6 Suits",
            "timed": true,
            "timeBase": 60,
            "timePerTurn": 20,
            "speedrun": false,
            "cardCycle": false,
            "deckPlays": false,
            "emptyClues": false,
            "oneExtraCard": false,
            "oneLessCard": false,
            "allOrNothing": false,
            "detrimentalCharacters": false
        }"#,
        )
        .unwrap();
        assert_eq!(options.num_players, 4);
        assert_eq!(options.starting_player, 1);
        assert_eq!(options.variant_id, 10);
        assert_eq!(options.variant_name, "6 Suits");
        assert!(options.timed);
        assert_eq!(options.time_base, 60);
        assert_eq!(options.time_per_turn, 20);
    }

    #[test]
    fn test_deserialize_table_list() {
        let tables: Vec<Table> = serde_json::from_str(TABLE_LIST_PAYLOAD).unwrap();
        assert_eq!(tables.len(), 1);
        let table = &tables[0];
        assert_eq!(table.id, 12345);
        assert_eq!(table.name, "Test Table");
        assert!(!table.password_protected);
        assert!(!table.joined);
        assert!(!table.running);
        assert_eq!(table.players, vec!["Alice", "Bob", "Cathy"]);
        assert_eq!(table.variant, "No Variant");
        assert_eq!(table.spectators.len(), 0);
    }

    #[test]
    fn test_deserialize_table() {
        let table: Table = serde_json::from_str(TABLE_PAYLOAD).unwrap();
        assert_eq!(table.id, 12345);
        assert_eq!(table.name, "Updated Table");
        assert!(table.joined);
        assert!(table.running);
        assert_eq!(table.progress, 50);
        assert_eq!(table.spectators.len(), 1);
        let spectator = &table.spectators[0];
        assert_eq!(spectator.name, "Dave");
        assert_eq!(spectator.shadowing_player_index, 0);
        assert_eq!(spectator.shadowing_player_username, "Alice");
    }

    #[test]
    fn test_deserialize_chat_message() {
        let msg: ChatMessage = serde_json::from_str(CHAT_MESSAGE_PAYLOAD).unwrap();
        assert_eq!(msg.msg, "!join");
        assert_eq!(msg.who, "someuser");
        assert!(!msg.discord);
        assert!(!msg.server);
        assert_eq!(msg.room, "lobby");
    }

    #[test]
    fn test_deserialize_game_action_list() {
        let data: GameActionListData = serde_json::from_str(GAME_ACTION_LIST_PAYLOAD).unwrap();
        assert_eq!(data.table_id, 37956);
        assert_eq!(data.list.len(), 3);

        match &data.list[0] {
            GameActionData::Draw(draw) => {
                assert_eq!(draw.player_index, 0);
                assert_eq!(draw.order, 0);
                assert_eq!(draw.suit_index, 4);
                assert_eq!(draw.rank, 4);
            }
            _ => panic!("Expected Draw action"),
        }

        match &data.list[2] {
            GameActionData::Turn(turn) => {
                assert_eq!(turn.num, 10);
                assert_eq!(turn.current_player_index, 2);
            }
            _ => panic!("Expected Turn action"),
        }
    }

    #[test]
    fn test_deserialize_play_action() {
        let action: GameAction = serde_json::from_str(GAME_ACTION_PLAY).unwrap();
        assert_eq!(action.table_id, 12345);
        match action.action {
            GameActionData::Play(play) => {
                assert_eq!(play.player_index, 1);
                assert_eq!(play.order, 42);
                assert_eq!(play.suit_index, 2);
                assert_eq!(play.rank, 2);
            }
            _ => panic!("Expected Play action"),
        }
    }

    #[test]
    fn test_deserialize_clue_action() {
        let action: GameAction = serde_json::from_str(GAME_ACTION_CLUE).unwrap();
        assert_eq!(action.table_id, 12345);
        match action.action {
            GameActionData::Clue(clue) => {
                assert_eq!(clue.giver, 0);
                assert_eq!(clue.target, 1);
                assert_eq!(clue.turn, 5);
                assert_eq!(clue.clue.clue_type, 2);
                assert_eq!(clue.clue.value, 0);
                assert_eq!(clue.list, vec![0, 2]);
            }
            _ => panic!("Expected Clue action"),
        }
    }

    #[test]
    fn test_deserialize_discard_action() {
        let action: GameAction = serde_json::from_str(GAME_ACTION_DISCARD).unwrap();
        assert_eq!(action.table_id, 12345);
        match action.action {
            GameActionData::Discard(discard) => {
                assert_eq!(discard.player_index, 2);
                assert_eq!(discard.order, 10);
                assert_eq!(discard.suit_index, 1);
                assert_eq!(discard.rank, 3);
                assert!(!discard.failed);
            }
            _ => panic!("Expected Discard action"),
        }
    }

    #[test]
    fn test_game_action_data_action_type() {
        let draw = GameActionData::Draw(DrawActionData {
            player_index: 0,
            order: 0,
            suit_index: 0,
            rank: 1,
        });
        assert_eq!(draw.action_type(), GameActionType::Draw);

        let play = GameActionData::Play(PlayActionData {
            player_index: 0,
            order: 0,
            suit_index: 0,
            rank: 1,
        });
        assert_eq!(play.action_type(), GameActionType::Play);

        let discard = GameActionData::Discard(DiscardActionData {
            player_index: 0,
            order: 0,
            suit_index: 0,
            rank: 1,
            failed: false,
        });
        assert_eq!(discard.action_type(), GameActionType::Discard);

        let clue = GameActionData::Clue(ClueActionData {
            clue: crate::external::hanablive::dto::instruction::game_action_data::ClueValue {
                clue_type: 2,
                value: 0,
            },
            giver: 0,
            list: vec![],
            target: 1,
            turn: 0,
        });
        assert_eq!(clue.action_type(), GameActionType::Clue);

        let turn = GameActionData::Turn(TurnActionData {
            num: 0,
            current_player_index: 0,
        });
        assert_eq!(turn.action_type(), GameActionType::Turn);

        let status = GameActionData::Status(StatusActionData {
            clues: 8,
            score: 0,
            max_score: 30,
        });
        assert_eq!(status.action_type(), GameActionType::Status);

        let strike = GameActionData::Strike(StrikeActionData {
            num: 1,
            turn: 0,
            order: 0,
        });
        assert_eq!(strike.action_type(), GameActionType::Strike);
    }

    #[test]
    fn test_deserialize_variants_metadata() {
        let variants: Vec<VariantMetadata> = serde_json::from_str(VARIANTS_METADATA).unwrap();
        assert_eq!(variants.len(), 2);

        let no_variant = &variants[0];
        assert_eq!(no_variant.id, 0);
        assert_eq!(no_variant.new_id, "no_variant");
        assert_eq!(no_variant.name, "No Variant");
        assert_eq!(no_variant.suits.len(), 5);
        assert_eq!(no_variant.stack_size, 5);
        assert_eq!(no_variant.clue_ranks, vec![1, 2, 3, 4, 5]);
        assert!(!no_variant.up_or_down);

        let six_suits = &variants[1];
        assert_eq!(six_suits.id, 1);
        assert_eq!(six_suits.name, "6 Suits");
        assert_eq!(six_suits.suits.len(), 6);
    }

    #[test]
    fn test_deserialize_suits_metadata() {
        let suits: Vec<SuitMetadata> = serde_json::from_str(SUITS_METADATA).unwrap();
        assert_eq!(suits.len(), 2);

        let red = &suits[0];
        assert_eq!(red.name, "Red");
        assert_eq!(red.id, "red");
        assert_eq!(red.abbreviation, "r");
        assert!(!red.prism);
        assert!(!red.one_of_each);

        let yellow = &suits[1];
        assert_eq!(yellow.name, "Yellow");
        assert_eq!(yellow.id, "yellow");
    }

    #[test]
    fn test_table_join_instruction() {
        let instruction = TableJoin { table_id: 42 };
        let msg = instruction.to_websocket_message();
        assert!(msg.starts_with("tableJoin "));
        let payload: serde_json::Value = serde_json::from_str(&msg["tableJoin ".len()..]).unwrap();
        assert_eq!(payload["tableID"], 42);
        assert!(payload.get("password").is_none());
    }

    #[test]
    fn test_password_protected_table_join_instruction() {
        let instruction = PasswordProtectedTableJoin {
            table_id: 42,
            password: "secret".to_string(),
        };
        let msg = instruction.to_websocket_message();
        assert!(msg.starts_with("tableJoin "));
        let payload: serde_json::Value = serde_json::from_str(&msg["tableJoin ".len()..]).unwrap();
        assert_eq!(payload["tableID"], 42);
        assert_eq!(payload["password"], "secret");
    }

    #[test]
    fn test_get_game_info1_instruction() {
        let instruction = GetGameInfo1 { table_id: 123 };
        let msg = instruction.to_websocket_message();
        assert!(msg.starts_with("getGameInfo1 "));
        let payload: serde_json::Value =
            serde_json::from_str(&msg["getGameInfo1 ".len()..]).unwrap();
        assert_eq!(payload["tableID"], 123);
    }

    #[test]
    fn test_get_game_info2_instruction() {
        let instruction = GetGameInfo2 { table_id: 456 };
        let msg = instruction.to_websocket_message();
        assert!(msg.starts_with("getGameInfo2 "));
        let payload: serde_json::Value =
            serde_json::from_str(&msg["getGameInfo2 ".len()..]).unwrap();
        assert_eq!(payload["tableID"], 456);
    }

    #[test]
    fn test_deserialize_status_action() {
        let json = r#"{
            "tableID": 1,
            "action": {
                "type": "status",
                "clues": 8,
                "score": 15,
                "maxScore": 30
            }
        }"#;
        let action: GameAction = serde_json::from_str(json).unwrap();
        match action.action {
            GameActionData::Status(status) => {
                assert_eq!(status.clues, 8);
                assert_eq!(status.score, 15);
                assert_eq!(status.max_score, 30);
            }
            _ => panic!("Expected Status action"),
        }
    }

    #[test]
    fn test_deserialize_strike_action() {
        let json = r#"{
            "tableID": 1,
            "action": {
                "type": "strike",
                "num": 2,
                "turn": 15,
                "order": 42
            }
        }"#;
        let action: GameAction = serde_json::from_str(json).unwrap();
        match action.action {
            GameActionData::Strike(strike) => {
                assert_eq!(strike.num, 2);
                assert_eq!(strike.turn, 15);
                assert_eq!(strike.order, 42);
            }
            _ => panic!("Expected Strike action"),
        }
    }

    #[test]
    fn test_deserialize_rank_clue_action() {
        let json = r#"{
            "tableID": 1,
            "action": {
                "type": "clue",
                "clue": {
                    "type": 3,
                    "value": 5
                },
                "giver": 1,
                "list": [1, 3],
                "target": 2,
                "turn": 8
            }
        }"#;
        let action: GameAction = serde_json::from_str(json).unwrap();
        match action.action {
            GameActionData::Clue(clue) => {
                assert_eq!(clue.clue.clue_type, 3);
                assert_eq!(clue.clue.value, 5);
                assert_eq!(clue.giver, 1);
                assert_eq!(clue.target, 2);
            }
            _ => panic!("Expected Clue action"),
        }
    }

    #[test]
    fn test_deserialize_failed_discard_action() {
        let json = r#"{
            "tableID": 1,
            "action": {
                "type": "discard",
                "playerIndex": 0,
                "order": 5,
                "suitIndex": 2,
                "rank": 4,
                "failed": true
            }
        }"#;
        let action: GameAction = serde_json::from_str(json).unwrap();
        match action.action {
            GameActionData::Discard(discard) => {
                assert!(discard.failed);
            }
            _ => panic!("Expected Discard action"),
        }
    }
}
